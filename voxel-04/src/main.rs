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
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use na::{Point2, Point3, Translation3, Vector3};
use rand::seq::SliceRandom;
use rand::Rng;

use actor::Actor;
use world::*;
use std::collections::HashMap;

fn add_voxel(
    group: &mut SceneNode,
    x: f32,
    height: f32,
    z: f32,
    color_index: i16,
) -> kiss3d::scene::SceneNode {
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
    let mut uvs: Vec<na::Point2<f32>> = vec![];

    let mut add_quad = |i0, i1, i2, i3, nx, ny, nz| {
        let i = points.len() as u16;
        points.push(base[i0 as usize].clone());
        points.push(base[i1 as usize].clone());
        points.push(base[i2 as usize].clone());
        points.push(base[i3 as usize].clone());

        let q = 0.0 / 16.0;
        let u = ((color_index % 2) as f32) / 2.0 + q;
        let v = ((color_index / 2) as f32) / 2.0 + q;
        let duv = 8.0 / 16.0 - q;
        uvs.push(Point2::new(u, v));
        uvs.push(Point2::new(u + duv, v));
        uvs.push(Point2::new(u + duv, v + duv));
        uvs.push(Point2::new(u, v + duv));

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

    let mesh = kiss3d::resource::Mesh::new(points, indices, None, Some(uvs), false);

    let data = std::rc::Rc::new(std::cell::RefCell::new(mesh));
    let mut c = group.add_mesh(data, Vector3::new(1.0, 1.0, 1.0));
    c.set_local_translation(Translation3::new(x, 0.0, z));
    c
}

enum Action {
    Move { x: i32, y: i32 },
}

fn sync_actor_node(window: &mut Window, world_map: &WorldMap, actor: &mut Actor) {
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
    let z = world_map.height(x, y);
    //let i = actor.y as usize * REGION_SIZE + actor.x as usize;
    //let z = world.heightmap[i] as f32;
    let t = Translation3::new(x as f32, z as f32 + 0.5, y as f32);
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

struct ChunkGeom {
    sync_id: u64,
    group: SceneNode,
}

struct WorldMapGeometry {
    chunks : HashMap<(i64, i64, i64), ChunkGeom>,
}

fn sync_world_map(
    wmg: &mut WorldMapGeometry,
    window: &mut Window,
    world: &World,
    mut texture_manager: &mut TextureManager,
) {
    /*if wmg.sync_id == world.world_map.sync_id {
        return;
    }
    wmg.sync_id = world.world_map.sync_id;*/

    let t = vec![
        (0, 0, 0),
        (32, 0, 0),
        (32, 32, 0),
        (0, 32, 0),
    ];
    for (x,y,z) in t {
        let world_sync_id = world.world_map.chunk_sync_id(x, y, z);

        let mut opt = wmg.chunks.get_mut(&(x,y,z));
        let mut opt = opt.as_mut();
        if opt.is_some() {
            let chunk = opt.unwrap();
            if chunk.sync_id == world_sync_id {
                continue;
            }
            chunk.group.unlink();
        }
        


        let mut group = window.add_group();
        chunk(&mut group, x, y, z, 32, &world.world_map, &mut texture_manager);
        wmg.chunks.insert((x, y, z), ChunkGeom{ 
            sync_id : world_sync_id,
            group : group,
        });
    }
}

extern crate image;

use crate::image::{GenericImage, Pixel};
use kiss3d::resource::Texture;
use kiss3d::resource::{Mesh, TextureManager};

fn make_texture() -> TextureManager { 
    let mut tm = TextureManager::new();
    tm.add(std::path::Path::new("./texture.png"), "tiles");
    tm
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

    let mut texture_manager = make_texture();

    // NPCs
    let mut rng = rand::thread_rng();
    let mut world = World::new(&mut rng);
    let mut wmg = WorldMapGeometry {
        chunks : HashMap::new(),
    };

    {
        let mut player = Actor::new();
        player.name = "Kestrel".to_string();
        player.occupation = Box::new(crate::occupations::Avatar::new());
        player.occupation.init(&mut player.state);
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
    for _ in 0..32 {
        let name = names.choose(&mut rng);
        let mut actor = Actor::new();
        actor.name = name.unwrap().to_string();
        actor.state.set_position(
            rng.gen_range(0, REGION_SIZE) as i64,
            rng.gen_range(0, REGION_SIZE) as i64,
        );

        actor.occupation = match rng.gen_range(0, 100) {
            0..=20 => Box::new(crate::occupations::Farmer::new()),
            _ => Box::new(crate::occupations::Mindlessness::new()),
        };
        actor.occupation.init(&mut actor.state);

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

    let mut time_total = std::time::Duration::new(0, 0);
    let mut time_graphics_update = std::time::Duration::new(0, 0);
    let mut time_strategies = std::time::Duration::new(0, 0);
    let mut time_graphics_world_map = std::time::Duration::new(0, 0);

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
        let start_time = std::time::Instant::now();
        if let Some((actor, other_actors)) = world.actors.split_first_mut() {
            let mut i = 0;
            while i < other_actors.len() {
                actor.occupation.update(game_time);
                if actor.strategy.is_none() {
                    let strategy = actor.occupation.generate_strategy();
                    actor.strategy = Some(strategy);
                }

                let context = occupation::Context {
                    game_time,
                    map: &mut world.world_map,
                    actor_state: &mut actor.state,
                    other_actors: other_actors,
                };
                actor.strategy.as_mut().unwrap().update(context);

                std::mem::swap(actor, &mut other_actors[i]);
                i += 1;
            }
        }
        world.player_index = (world.player_index + 1) % world.actors.len();
        time_strategies += start_time.elapsed();

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
                    println!("Going to move {} to {}, {}", player_name, nx, ny);

                    if let Some(index) = world.actor_at_tile(nx, ny) {
                        let actor = &mut world.actors[index];
                        let occupation_name = actor.occupation.name();

                        let key = "Hello".to_string();
                        if !actor.shortterm_memory.contains_key(&key) {
                            let s = format!(
                                "{player_name} says to the {occupation}, \"Hello, {name}.\"",
                                player_name = player_name,
                                name = actor.name,
                                occupation = occupation_name,
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
        let start_time = std::time::Instant::now();

        let start_time_wm = std::time::Instant::now();
        sync_world_map(&mut wmg, &mut window, &world, &mut texture_manager);
        time_graphics_world_map += start_time_wm.elapsed();

        let actor_list = &mut world.actors;
        for actor in actor_list {
            sync_actor_node(&mut window, &world.world_map, actor);
        }
        time_graphics_update += start_time.elapsed();

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
                &format!(
                    "FPS: {:.1} / {:.1}% / {:.1}% / {:.1}%",
                    fps,
                    percentage(time_graphics_update, time_total),
                    percentage(time_graphics_world_map, time_total),
                    percentage(time_strategies, time_total),
                )[..],
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
        time_total += timestamp.elapsed();
    }
}

fn percentage(part: std::time::Duration, whole: std::time::Duration) -> f64 {
    let f = part.as_nanos() as f64 / whole.as_nanos() as f64;
    f * 100.0
}

fn chunk(group: &mut SceneNode, ox: i64, oy: i64, oz: i64, chunk_size: i64, world_map: &WorldMap, texture_manager: &mut TextureManager) {

    // Points on a unit cube
    let e = -0.5f32;
    let s = 0.5f32;
    let base: Vec<Point3<f32>> = vec![
        Point3::new(e, e, e),
        Point3::new(e, s, e),
        Point3::new(s, s, e),
        Point3::new(s, e, e),
        Point3::new(e, e, s),
        Point3::new(s, e, s),
        Point3::new(s, s, s),
        Point3::new(e, s, s),
    ];
    let face_lists: Vec<Vec<usize>> = vec![
        vec![4, 7, 1, 0], // -X        
        vec![3, 2, 6, 5], // X               

        vec![0, 3, 5, 4], // -Y
        vec![7, 6, 2, 1], // Y

        vec![3, 0, 1, 2], // -Z
        vec![4, 5, 6, 7], // Z
    ];

    // Vectors for the normal directions
    let normal_list: Vec<Vector3<f32>> = vec![
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, 1.0),
    ];

    // Mesh arrays
    let mut points = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut uvs: Vec<na::Point2<f32>> = vec![];

    let mut add_quad = |cx, cy, cz, face_id: usize, color_index| {

        let i = points.len() as u16;
        for pi in &face_lists[face_id] {
            let mut pt = base[*pi].clone(); 
            pt.x += cx as f32;
            pt.y += cy as f32;
            pt.z += cz as f32;
            points.push(pt);
        }
        indices.push(Point3::new(i + 0, i + 1, i + 2));
        indices.push(Point3::new(i + 0, i + 2, i + 3));

        normals.push(normal_list[face_id].clone());

        let ci = color_index - 1;
        let u = ((ci % 2) as f32) / 2.0;
        let v = ((ci / 2) as f32) / 2.0;
        let duv = 8.0 / 16.0;
        uvs.push(Point2::new(u, v));
        uvs.push(Point2::new(u + duv, v));
        uvs.push(Point2::new(u + duv, v + duv));
        uvs.push(Point2::new(u, v + duv));
    };

    // Shorthand to look up tile values
    let lookup = |cx, cy, cz| -> u8 {
        if cx < 0 || cx >= chunk_size || cy < 0 || cy >= chunk_size || cz < 0 || cz >= chunk_size {
            0
        } else {
            world_map.tile3(ox + cx, oy + cy, oz + cz)
        }
    };

    for cx in 0..chunk_size {
        for cy in 0..chunk_size {
            for cz in 0..chunk_size {
                let color_index = lookup(cx, cy, cz);
                if color_index == 0 {
                    continue;                
                }

                // Swap Y and Z to match kiss3d coordinate system
                // Subtract 1 from kiss3d y coordinate system to offset heightmap

                // World X
                if lookup(cx + 1, cy, cz) == 0 {  
                    add_quad(cx, cz - 1, cy, 1, color_index);
                }
                if lookup(cx - 1, cy, cz) == 0 {  
                    add_quad(cx, cz - 1, cy, 0, color_index);
                }

                // World Y
                if lookup(cx, cy + 1, cz) == 0 {  
                    add_quad(cx, cz - 1, cy, 5, color_index);
                }
                if lookup(cx, cy - 1, cz) == 0 {  
                    add_quad(cx, cz - 1, cy, 4, color_index);
                }

                // World Z
                if lookup(cx, cy, cz + 1) == 0 {  
                    add_quad(cx, cz - 1, cy, 3, color_index);
                }
                if lookup(cx, cy, cz - 1) == 0 {  
                    add_quad(cx, cz - 1, cy, 2, color_index);
                }
            }
        }
    }

    let mesh = kiss3d::resource::Mesh::new(points, indices, None, Some(uvs), false);

    let data = std::rc::Rc::new(std::cell::RefCell::new(mesh));
    let scale = Vector3::new(1.0, 1.0, 1.0);
    let translation = Translation3::new(ox as f32, oz as f32, oy as f32);
    let mut c = group.add_mesh(data, scale);
    c.set_local_translation(translation);

    if let Some(texture) = texture_manager.get("tiles") {
        c.set_texture(texture);
    }
}
