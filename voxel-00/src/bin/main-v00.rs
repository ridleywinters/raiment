extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::light::Light;
use kiss3d::window::Window;
use na::{Point3, Translation3};
use rand::Rng;
use rand::seq::SliceRandom;

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

struct Kestrel {
    x: i64,
    y: i64,

    dirty: bool,
    node: Option<kiss3d::scene::SceneNode>,
}

struct Actor {
    x: i64,
    y: i64,
    name : String,

    dirty: bool,
    node: Option<kiss3d::scene::SceneNode>,
}

struct World {
    seed: usize,
    pub player: Kestrel,
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
            player: Kestrel {
                x: 0,
                y: 0,
                dirty: true,
                node: None,
            },
            actors: vec![],
            heightmap,
        }
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

fn main() {
    let mut window = Window::new_with_size("raiment: voxel-01", 800, 800);
    window.set_light(Light::StickToCamera);

    let mut camera = kiss3d::camera::FirstPerson::new_with_frustrum(
        70.0 * std::f32::consts::PI / 180.0,
        0.01,
        1000.0,
        Point3::new(
            REGION_SIZE as f32 * 0.8,
            REGION_SIZE as f32 * 0.6,
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

    world.player.x = rng.gen_range(0, REGION_SIZE as i64);
    world.player.y = rng.gen_range(0, REGION_SIZE as i64);

    let names = vec![ "Raelon", "Telenor", "Sentor", "Baaren", "Celinac", "Coplin", "Boran", "Ilia", "Kelis", "Elli", "Len" ];
    for _ in 0..8 {
        let x: usize = rng.gen_range(0, REGION_SIZE);
        let y: usize = rng.gen_range(0, REGION_SIZE);

        let name = names.choose(&mut rng);
        let actor = Actor {
            x: x as i64,
            y: y as i64,
            name : name.unwrap().to_string(),

            dirty: true,
            node: None,
        };
        if world.is_tile_empty(actor.x, actor.y) {
            println!("Adding {}", actor.name);
            world.actors.push(actor);
        } else {
            println!("Collision!");
        }
    }

    println!("Beginning render loop...");
    let mut last_move = 0;
    let move_threshold = 200;
    while window.render_with_camera(&mut camera) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH) 
            .expect("ok")
            .as_millis();

        //
        // Check input
        //
        if timestamp - last_move > move_threshold {
            if window.get_key(kiss3d::event::Key::Left) == kiss3d::event::Action::Press {
                if world.is_tile_empty(world.player.x + 1, world.player.y) {
                    world.player.x += 1;
                    world.player.dirty = true;
                    last_move = timestamp;
                }
            }
            if window.get_key(kiss3d::event::Key::Right) == kiss3d::event::Action::Press {
                if world.is_tile_empty(world.player.x - 1, world.player.y) {
                    world.player.x -= 1;
                    world.player.dirty = true;
                    last_move = timestamp;
                }
            }

            if window.get_key(kiss3d::event::Key::Up) == kiss3d::event::Action::Press {
                if world.is_tile_empty(world.player.x, world.player.y + 1) {
                    world.player.y += 1;
                    world.player.dirty = true;
                    last_move = timestamp;
                }
            }
            if window.get_key(kiss3d::event::Key::Down) == kiss3d::event::Action::Press {
                if world.is_tile_empty(world.player.x, world.player.y - 1) {
                    world.player.y -= 1;
                    world.player.dirty = true;
                    last_move = timestamp;
                }
            }
        }

        //
        // State change
        //
        if world.player.dirty {
            let x = world.player.x;
            let y = world.player.y;
            let i = (y * REGION_SIZE as i64 + x) as usize;
            let z = world.heightmap[i] as f32;
            let t = Translation3::new(x as f32, z, y as f32);

            if world.player.node.is_none() {
                world.player.node = Some(add_cylinder(
                    &mut window,
                    x as f32,
                    z as f32,
                    y as f32,
                    0.5,
                    0.5,
                    0.6,
                ));
            }
            match &mut world.player.node {
                Some(node) => {
                    node.set_local_translation(t);
                }
                None => {}
            }

            /*let eye = (camera as kiss3d::camera::Camera).eye();
            camera.look_at(eye, Point3::new(x as f32, z, y as f32));*/

            world.player.dirty = false;
        }

        for actor in &mut world.actors {
            let x = actor.x;
            let y = actor.y;
            let i = actor.y as usize * REGION_SIZE + actor.x as usize;
            let z = world.heightmap[i] as f32;
            let t = Translation3::new(x as f32, z, y as f32);

            if actor.node.is_none() {
                let c = add_cylinder(&mut window, x as f32, z, y as f32, 1.0, 0.0, 0.0);
                actor.node = Some(c);
            }
            match &mut actor.node {
                Some(node) => {
                    node.set_local_translation(t);
                }
                None => {}
            }

            if actor.dirty {
                actor.dirty = false;
            }
        }
    }
}
