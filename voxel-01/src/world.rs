use rand::Rng;
use crate::actor::Actor;


pub const REGION_SIZE: usize = 32;

pub struct World {
    seed: usize,
    pub player: Actor,
    pub actors: Vec<Actor>,

    pub heightmap: [i32; REGION_SIZE * REGION_SIZE],
}

impl World {
    pub fn new(rng: &mut rand::rngs::ThreadRng) -> World {
        println!("Building heightmap...");
        let offset_x = rng.gen_range(0.0, 100.0);
        let offset_y = rng.gen_range(0.0, 100.0);
        let scale_x = rng.gen_range(0.5, 1.5);
        let scale_y = rng.gen_range(0.5, 1.5);
        println!("{} {}", offset_x, offset_y);
        let mut heightmap: [i32; REGION_SIZE * REGION_SIZE] = [0; REGION_SIZE * REGION_SIZE];
        for y in 0..REGION_SIZE {
            for x in 0..REGION_SIZE {
                let a = offset_x
                    + (x as f32 * scale_x) * std::f32::consts::PI / (REGION_SIZE as f32 / 2.0);
                let b = offset_y
                    + (y as f32 * scale_y) * std::f32::consts::PI / (REGION_SIZE as f32 / 4.0);
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

    pub fn actor_at_tile(&self, x: i64, y: i64) -> Option<usize> {
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

    pub fn is_tile_empty(&self, x: i64, y: i64) -> bool {
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