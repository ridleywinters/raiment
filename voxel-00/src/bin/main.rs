extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::light::Light;
use kiss3d::window::Window;
use na::{Point3, Translation3};
use rand::seq::SliceRandom;
use rand::Rng;

const REGION_SIZE: usize = 32;
const LAYER_HEIGHT: usize = 4;

struct Cell {
    solid: bool,
}

struct RegionZLayer {
    cells: [Cell; REGION_SIZE * REGION_SIZE * LAYER_HEIGHT],
}

struct Region {
    layers: [Option<RegionZLayer>; REGION_SIZE / LAYER_HEIGHT],
}

fn add_voxel(
    window: &mut Window,
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    g: f32,
    b: f32,
) -> kiss3d::scene::SceneNode {
    let mut c = window.add_cube(1.0, 1.0, 1.0);
    c.set_color(r, g, b);
    c.set_local_translation(Translation3::new(x, y, z));
    c
}

fn add_cylinder(
    window: &mut Window,
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    g: f32,
    b: f32,
) -> kiss3d::scene::SceneNode {
    let mut c = window.add_cylinder(0.4, 0.8);
    c.set_color(r, g, b);
    c.set_local_translation(Translation3::new(x, y, z));
    c
}

struct Actor {
    sync_id: u64,
    x: i64,
    y: i64,
    name: String,
    r: f32,
    g: f32,
    b: f32,

    node_sync_id: u64,
    node: Option<kiss3d::scene::SceneNode>,
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.sync_id)
    }
}

impl Actor {
    fn new() -> Actor {
        Actor {
            // Start at one so any dependent sync_id's are out-of-sync by default
            sync_id: 1,

            x: 0,
            y: 0,
            name: String::new(),
            r: 0.0,
            g: 0.0,
            b: 0.0,

            node_sync_id: 0,
            node: None,
        }
    }

    fn set_position(&mut self, x: i64, y: i64) {
        self.x = x;
        self.y = y;
        self.sync_id += 1;
    }

    fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.r = r;
        self.g = g;
        self.b = b;
        self.sync_id += 1;
    }
}

struct World {
    seed: usize,
    pub player: Actor,
    pub actors: Vec<Actor>,

    pub heightmap: [i32; REGION_SIZE * REGION_SIZE],
}

impl World {
    fn new() -> World {
        println!("Building heightmap...");
        let mut heightmap: [i32; REGION_SIZE * REGION_SIZE] = [0; REGION_SIZE * REGION_SIZE];
        for y in 0..REGION_SIZE {
            for x in 0..REGION_SIZE {
                let a = (x as f32) * std::f32::consts::PI / (REGION_SIZE as f32 / 2.0);
                let b = (y as f32) * std::f32::consts::PI / (REGION_SIZE as f32 / 4.0);
                let z = 2.0 * ((a.sin() + 0.5) + (b.cos() + 0.5));
                let i = y * REGION_SIZE + x;
                heightmap[i] = (z as i32).max(1);
            }
        }

        World {
            seed: 52329,
            player: Actor::new(),
            actors: vec![],
            heightmap,
        }
    }
    fn actor_at_tile(&self, x: i64, y: i64) -> Option<usize> {
        if x < 0 || x >= REGION_SIZE as i64 || y < 0 || y >= REGION_SIZE as i64 {
            return None;
        }
        if self.player.x == x && self.player.y == y {
            return None;
        }

        for i in 0..self.actors.len() {
            let actor = &self.actors[i];
            if actor.x == x && actor.y == y {
                return Some(i);
            }
        }
        None
    }

    fn is_tile_empty(&self, x: i64, y: i64) -> bool {
        if x < 0 || x >= REGION_SIZE as i64 {
            return false;
        }
        if y < 0 || y >= REGION_SIZE as i64 {
            return false;
        }
        if self.player.x == x && self.player.y == y {
            return false;
        }
        for actor in &self.actors {
            if actor.x == x && actor.y == y {
                return false;
            }
        }

        true
    }
}

enum Action {
    Move { x: i32, y: i32 },
}

fn sync_actor_node(window: &mut Window, z: f32, actor: &mut Actor) {
    //
    // Check if the graphics node is already in sync
    //
    if actor.sync_id == actor.node_sync_id {
        return;
    }
    actor.node_sync_id = actor.sync_id;

    //
    // Graphics object not yet created
    //
    if actor.node.is_none() {
        let node = window.add_cylinder(0.4, 0.8);
        actor.node = Some(node);
    }
    let node = actor.node.as_mut().unwrap();

    //
    // Update position
    //
    let x = actor.x;
    let y = actor.y;
    //let i = actor.y as usize * REGION_SIZE + actor.x as usize;
    //let z = world.heightmap[i] as f32;
    let t = Translation3::new(x as f32, z, y as f32);
    node.set_local_translation(t);

    //
    // Update color
    //
    node.set_color(actor.r, actor.g, actor.b);
}

fn main() {
    let mut window = Window::new_with_size("raiment: voxel-01", 800, 800);
    window.set_light(Light::StickToCamera);

    let mut camera = kiss3d::camera::FirstPerson::new_with_frustrum(
        70.0 * std::f32::consts::PI / 180.0,
        0.01,
        1000.0,
        Point3::new(
            REGION_SIZE as f32 * 0.4,
            REGION_SIZE as f32 * 0.8,
            REGION_SIZE as f32 * -0.1,
        ),
        Point3::new(REGION_SIZE as f32 / 2.0, 0.0, REGION_SIZE as f32 / 2.0),
    );
    camera.rebind_up_key(Some(kiss3d::event::Key::W));
    camera.rebind_down_key(Some(kiss3d::event::Key::S));
    camera.rebind_left_key(Some(kiss3d::event::Key::A));
    camera.rebind_right_key(Some(kiss3d::event::Key::D));

    // NPCs
    let mut rng = rand::thread_rng();

    let mut world = World::new();

    println!("Populating geometry...");
    for y in 0..REGION_SIZE {
        for x in 0..REGION_SIZE {
            let i = y * REGION_SIZE + x;
            for z in 0..world.heightmap[i] {
                add_voxel(&mut window, x as f32, z as f32, y as f32, 0.2, 0.6, 0.44);
            }
        }
    }

    world.player.name = "Kestrel".to_string();
    world.player.set_color(0.2, 0.5, 0.8);
    world.player.set_position(
        rng.gen_range(0, REGION_SIZE as i64),
        rng.gen_range(0, REGION_SIZE as i64),
    );

    let names = vec![
        "Raelon", "Telenor", "Sentor", "Baaren", "Celinac", "Coplin", "Boran", "Ilia", "Kelis",
        "Elli", "Len",
    ];
    for _ in 0..8 {
        let name = names.choose(&mut rng);
        let mut actor = Actor::new();
        actor.name = name.unwrap().to_string();
        actor.set_color(0.3, 0.3, 0.3);
        actor.set_position(
            rng.gen_range(0, REGION_SIZE) as i64,
            rng.gen_range(0, REGION_SIZE) as i64,
        );
        if world.is_tile_empty(actor.x, actor.y) {
            println!("Adding {}", actor.name);
            world.actors.push(actor);
        } else {
            println!("Collision!");
        }
    }

    println!("Beginning render loop...");
    let mut last_move = 0;

    while window.render_with_camera(&mut camera) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("ok")
            .as_millis();

        let mut action_queue = Vec::new();

        //
        // Check input
        //
        // This abstraction decouples the input mechanism from the Action taken. This allows
        // for everything from customized key maps to automated scripts producing actions
        // and the rest of the engine is unaffected.
        //
        if window.get_key(kiss3d::event::Key::Left) == kiss3d::event::Action::Press {
            action_queue.push(Action::Move { x: 1, y: 0 });
        }
        if window.get_key(kiss3d::event::Key::Right) == kiss3d::event::Action::Press {
            action_queue.push(Action::Move { x: -1, y: 0 });
        }

        if window.get_key(kiss3d::event::Key::Up) == kiss3d::event::Action::Press {
            action_queue.push(Action::Move { x: 0, y: 1 });
        }
        if window.get_key(kiss3d::event::Key::Down) == kiss3d::event::Action::Press {
            action_queue.push(Action::Move { x: 0, y: -1 });
        }

        //
        // Process Actions for this frame
        //
        for action in action_queue.into_iter() {
            match action {
                Action::Move { x, y } => {
                    // Throttle Move commands, discard when there are too many
                    if timestamp - last_move < 100 {
                        continue;
                    }

                    let nx = world.player.x + x as i64;
                    let ny = world.player.y + y as i64;
                    if world.is_tile_empty(nx, ny) {
                        world.player.set_position(nx, ny);
                        last_move = timestamp;
                    }

                    if let Some(index) = world.actor_at_tile(nx, ny) {
                        let actor = &mut world.actors[index];
                        println!("{} says, \"Hello, {}.\"", world.player.name, actor.name);
                        actor.set_color(1.0, 1.0, 0.0);
                    }
                }
            };
        }

        //
        // Graphics cache update
        //
        {
            let actor = &mut world.player;
            // TODO: can't pass world to the sync function due to mutablility, so
            // have this ugly workaround
            let i = actor.y as usize * REGION_SIZE + actor.x as usize;
            let z = world.heightmap[i] as f32;
            sync_actor_node(&mut window, z, actor);
        }

        let actor_list = &mut world.actors;
        for actor in actor_list {
            // TODO: can't pass world to the sync function due to mutablility, so
            // have this ugly workaround
            let i = actor.y as usize * REGION_SIZE + actor.x as usize;
            let z = world.heightmap[i] as f32;
            sync_actor_node(&mut window, z, actor);
        }
    }
}
