extern crate kiss3d;
extern crate nalgebra as na;

// TODO: I don't understand why these need to "forward declared"...I need to learn more
// about Rust
mod actor;
mod world;
mod occupation;
mod occupations;

use kiss3d::light::Light;
use kiss3d::window::Window;
use na::{Point2, Point3, Translation3};
use rand::seq::SliceRandom;
use rand::Rng;

use actor::Actor;
use world::*;

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
        let node = window.add_cylinder(0.4, 1.8);
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

struct Text {
    text: String,
    expiration: std::time::Instant,
}

impl Text {
    fn new(s: &str, duration: f32) -> Text {
        let d = std::time::Instant::now();
        let d2 = d
            .checked_add(std::time::Duration::from_millis(duration as u64))
            .unwrap();
        Text {
            text: s.to_string(),
            expiration: d2,
        }
    }
}

fn main() {
    let mut window = Window::new_with_size("raiment: voxel-02", 800, 800);
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
    let mut world = World::new(&mut rng);

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
        "Elli", "Len", "Bilric", "Rownal", "Cal", "Wern", "Lendole", "Ilabin",
        "Revor", "Edien", "Dien", "Cien", "Ance", "Anker", "Manker", "Isotel", "Isse",
        "Lince"
    ];
    for _ in 0..8 {
        let name = names.choose(&mut rng);
        let mut actor = Actor::new();
        actor.name = name.unwrap().to_string();
        actor.occupation_name = (if rng.gen_range(0, 8) as i32 == 0 {
            "farmer"
        } else {
            "vagabond"
        })
        .to_string();
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
    let mut last_move = std::time::Instant::now();

    let font = kiss3d::text::Font::default();

    let mut texts = Vec::new();
    texts.push(Text::new("Welcome!", 15_000.0));

    let mut game_time : u64 = 1000;

    while window.render_with_camera(&mut camera) {
        let timestamp = std::time::Instant::now();

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
        // Update internal simulations
        //
        for actor in &mut world.actors {
            let mut keys = Vec::new();
            for marker in &mut actor.shortterm_memory {
                if marker.1.checked_duration_since(timestamp) == None {
                    keys.push(marker.0.clone());
                }
            }
            for k in keys {
                actor.shortterm_memory.remove(&k);
            }
        }

        //
        // 
        //
        for actor in &mut world.actors {
            actor.occupation.update(game_time);

            if actor.strategy.is_none() {
                println!("Assigning strategy...");
                let strategy = actor.occupation.generate_strategy();
                actor.strategy = Some(strategy);
            }

            let strategy = actor.strategy.as_mut().unwrap();
            strategy.update(game_time);
        }

        //
        // Process Actions for this frame
        //
        for action in action_queue.into_iter() {
            match action {
                Action::Move { x, y } => {
                    // Throttle Move commands, discard when there are too many
                    if timestamp.duration_since(last_move).as_millis() < 100 {
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
                        if actor.occupation_name == "farmer" {
                        actor.set_color(1.0, 1.0, 0.0);
                        } else {
                            actor.set_color(0.5, 0.3, 0.3);
                        }

                        let key = "Hello".to_string();
                        if !actor.shortterm_memory.contains_key(&key) {
                            let s = format!(
                                "{player_name} says to the {occupation}, \"Hello, {name}.\"",
                                player_name = world.player.name,
                                name = actor.name,
                                occupation = actor.occupation_name,
                            );
                            texts.push(Text::new(&s[..], 5_000.0));
                            actor.shortterm_memory.insert(
                                key,
                                timestamp
                                    .checked_add(std::time::Duration::from_millis(3000))
                                    .unwrap(),
                            );
                        }
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

        //
        // Text overlay (immediate render)
        //
        let mut live = Vec::new();
        let mut y_offset = 0.0;
        for text in &texts {
            if text.expiration.checked_duration_since(timestamp) == None {
                continue;
            }

            window.draw_text(
                &text.text,
                &Point2::new(12.0, y_offset),
                60.0,
                &font,
                &Point3::new(0.0, 0.6, 1.0),
            );
            y_offset += 80.0;

            live.push(text);
        }

        texts = texts
            .into_iter()
            .filter(|text| text.expiration.checked_duration_since(timestamp) != None)
            .collect::<Vec<_>>();

        game_time += 10;
    }
}
