extern crate kiss3d;
extern crate nalgebra as na;

// TODO: I don't understand why these need to "forward declared"...I need to learn more
// about Rust
mod actor;
mod occupation;
mod occupations;
mod tasks;
mod world;

use kiss3d::light::Light;
use kiss3d::window::Window;
use na::{Point2, Point3, Translation3, Vector3};
use rand::seq::SliceRandom;
use rand::Rng;

use actor::Actor;
use world::*;

const REGION_SIZE: usize = 32;

fn add_voxel(window: &mut Window, x: f32, height: f32, z: f32) -> kiss3d::scene::SceneNode {
    let s = 0.5f32;
    let e = -s;
    let y = e + (0.0 + height);
    let base: Vec<Point3<f32>> = vec![
        Point3::new(e, e, e),
        Point3::new(e, y, e),
        Point3::new(s, y, e),
        Point3::new(s, e, e),
        Point3::new(e, e, s),
        Point3::new(s, e, s),
        Point3::new(s, y, s),
        Point3::new(e, y, s),
    ];

    let mut points = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    let mut add_quad = |i0, i1, i2, i3, nx, ny, nz| {
        let i = points.len() as u16;
        points.push(base[i0 as usize].clone());
        points.push(base[i1 as usize].clone());
        points.push(base[i2 as usize].clone());
        points.push(base[i3 as usize].clone());

        normals.push(Vector3::new(nx, ny, nz));

        indices.push(Point3::new(i + 0, i + 1, i + 2));
        indices.push(Point3::new(i + 0, i + 2, i + 3));
    };

    add_quad(0, 3, 5, 4, 0.0, -1.0, 0.0);
    add_quad(7, 6, 2, 1, 0.0, 1.0, 0.0);

    add_quad(3, 2, 6, 5, 1.0, 0.0, 0.0);
    add_quad(4, 7, 1, 0, -1.0, 0.0, 0.0);

    add_quad(3, 0, 1, 2, 0.0, 0.0, -1.0);
    add_quad(4, 5, 6, 7, 0.0, 0.0, 1.0);

    let mesh = kiss3d::resource::Mesh::new(points, indices, None, None, false);

    let data = std::rc::Rc::new(std::cell::RefCell::new(mesh));
    let mut c = window.add_mesh(data, Vector3::new(1.0, 1.0, 1.0));
    c.set_local_translation(Translation3::new(x, 0.0, z));
    c
}

enum Action {
    Move { x: i32, y: i32 },
}

fn sync_actor_node(window: &mut Window, z: f32, actor: &mut Actor) {
    //
    // Check if the graphics node is already in sync
    //
    if actor.state.sync_id() == actor.node_sync_id {
        return;
    }
    actor.node_sync_id = actor.state.sync_id();

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
    let (x, y) = actor.state.position();
    //let i = actor.y as usize * REGION_SIZE + actor.x as usize;
    //let z = world.heightmap[i] as f32;
    let t = Translation3::new(x as f32, z + 0.5, y as f32);
    node.set_local_translation(t);

    //
    // Update color
    //
    let (r, g, b) = actor.state.color();
    node.set_color(r, g, b);
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

//
// Check input
//
// This abstraction decouples the input mechanism from the Action taken. This allows
// for everything from customized key maps to automated scripts producing actions
// and the rest of the engine is unaffected.
//
fn check_user_input(window: &Window, action_queue: &mut Vec<Action>) {
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

            let z = world.world_map.tiles[i];

            // Note that z and y are swapped here as y is up in the kiss3d
            // coordinate system
            let mut c = add_voxel(&mut window, x as f32, z as f32, y as f32);
            c.set_color(0.2, 0.6, 0.44);
        }
    }

    {
        let mut player = Actor::new();
        player.name = "Kestrel".to_string();
        player.state.set_color(0.2, 0.5, 0.8);
        player.state.set_position(
            rng.gen_range(0, REGION_SIZE as i64),
            rng.gen_range(0, REGION_SIZE as i64),
        );
        world.player_index = world.actors.len();
        world.actors.push(player);
    }

    let names = vec![
        "Raelon", "Telenor", "Sentor", "Baaren", "Celinac", "Coplin", "Boran", "Ilia", "Kelis",
        "Elli", "Len", "Bilric", "Rownal", "Cal", "Wern", "Lendole", "Ilabin", "Revor", "Edien",
        "Dien", "Cien", "Ance", "Anker", "Manker", "Isotel", "Isse", "Lince",
    ];
    for _ in 0..48 {
        let name = names.choose(&mut rng);
        let mut actor = Actor::new();
        actor.name = name.unwrap().to_string();
        actor.occupation_name = (if rng.gen_range(0, 8) as i32 == 0 {
            "farmer"
        } else {
            "vagabond"
        })
        .to_string();
        actor.state.set_color(0.3, 0.3, 0.3);
        actor.state.set_position(
            rng.gen_range(0, REGION_SIZE) as i64,
            rng.gen_range(0, REGION_SIZE) as i64,
        );
        
        actor.occupation = Box::new(crate::occupations::Mindlessness::new());
        let (x, y) = actor.state.position();
        if world.is_tile_empty(x, y) {
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

    let mut game_time: u64 = 0;
    let mut frame_number: u64 = 0;
    let time_real_start = std::time::Instant::now();

    while window.render_with_camera(&mut camera) {
        let timestamp = std::time::Instant::now();

        let mut action_queue = Vec::new();

        check_user_input(&window, &mut action_queue);

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
        // Update strategies
        //
        // NOTE: what's with this odd looking loop? The loop is refactored to allow us
        // to have a mutable reference to the current actor and immutable references to
        // all others. This is difficult to do "directly" since the borrow checker does
        // not (really?) understand borrows from part of an array/vector.
        //
        // TODO: there has to be a better way to do this.
        if let Some((actor, other_actors)) = world.actors.split_first_mut() {
            let mut i = 0;
            while i < other_actors.len() {
                actor.occupation.update(game_time);
                if actor.strategy.is_none() {
                    println!("Assigning strategy...");
                    let strategy = actor.occupation.generate_strategy();
                    actor.strategy = Some(strategy);
                }

                let context = occupation::Context {
                    game_time,
                    world_map: &world.world_map,
                    actor_state: &mut actor.state,
                    other_actors: other_actors,
                };
                actor.strategy.as_mut().unwrap().update(context);

                std::mem::swap(actor, &mut other_actors[i]);
                i += 1;
            }
        }
        world.player_index = (world.player_index + 1) % world.actors.len();

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

                    let (px, py) = world.actors[world.player_index].state.position();
                    let nx = px + x as i64;
                    let ny = py + y as i64;
                    if world.is_tile_empty(nx, ny) {
                        let player = &mut world.actors[world.player_index];
                        player.state.set_position(nx, ny);
                        last_move = timestamp;
                    }

                    let player_name = &world.actors[world.player_index].name.clone();
                    println!("Going to move {}", player_name);

                    if let Some(index) = world.actor_at_tile(nx, ny) {
                        let actor = &mut world.actors[index];
                        if actor.occupation_name == "farmer" {
                            actor.state.set_color(1.0, 1.0, 0.0);
                        } else {
                            actor.state.set_color(0.5, 0.3, 0.3);
                        }

                        let key = "Hello".to_string();
                        if !actor.shortterm_memory.contains_key(&key) {
                            let s = format!(
                                "{player_name} says to the {occupation}, \"Hello, {name}.\"",
                                player_name = player_name,
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
            let actor = &mut world.actors[world.player_index];
            // TODO: can't pass world to the sync function due to mutablility, so
            // have this ugly workaround
            let (x, y) = actor.state.position();
            let i = y as usize * REGION_SIZE + x as usize;
            let z = world.world_map.tiles[i] as f32;
            sync_actor_node(&mut window, z, actor);
        }

        let actor_list = &mut world.actors;
        for actor in actor_list {
            // TODO: can't pass world to the sync function due to mutablility, so
            // have this ugly workaround
            let (x, y) = actor.state.position();
            let i = y as usize * REGION_SIZE + x as usize;
            let z = world.world_map.tiles[i] as f32;
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

        if frame_number > 60 {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(time_real_start);
            let fps = (frame_number as f32) / elapsed.as_secs_f32();

            window.draw_text(
                &format!("FPS: {:.1}", fps)[..],
                &Point2::new(1200.0, 20.0),
                30.0,
                &font,
                &Point3::new(0.8, 0.8, 0.2),
            );
        }

        texts = texts
            .into_iter()
            .filter(|text| text.expiration.checked_duration_since(timestamp) != None)
            .collect::<Vec<_>>();

        game_time += 10;
        frame_number += 1;
    }
}
