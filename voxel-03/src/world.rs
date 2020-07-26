use crate::actor::Actor;
use rand::Rng;

pub const REGION_SIZE: usize = 32;

pub struct WorldMap {
    pub sync_id: u64,
    pub tiles: [i32; REGION_SIZE * REGION_SIZE],
}

impl WorldMap {
    pub fn new() -> Self {
        println!("Building heightmap...");
        let mut rng = rand::thread_rng();
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

        Self {
            sync_id: 1,
            tiles: heightmap,
        }
    }

    pub fn width(&self) -> i64 {
        REGION_SIZE as i64
    }

    pub fn length(&self) -> i64 {
        REGION_SIZE as i64
    }

    pub fn height(&self, x: i64, y: i64) -> i32 {
        let i = (y * self.width() + x) as usize;
        self.tiles[i]
    }

    pub fn set_height(&mut self, x: i64, y: i64, z: i64) {
        self.sync_id += 1;

        let i = (y * self.width() + x) as usize;
        println!(
            "Setting height at {},{} from {} -> {}",
            x, y, self.tiles[i], z
        );
        self.tiles[i] = z as i32;
    }

    pub fn is_tile_valid(&self, x: i64, y: i64) -> bool {
        if x < 0 || x >= REGION_SIZE as i64 {
            return false;
        }
        if y < 0 || y >= REGION_SIZE as i64 {
            return false;
        }
        true
    }
}

pub struct World {
    seed: usize,
    pub player_index: usize,
    pub actors: Vec<Actor>,
    pub world_map: WorldMap,
}

impl World {
    pub fn new(rng: &mut rand::rngs::ThreadRng) -> Self {
        Self {
            seed: 52329,
            player_index: 0,
            actors: vec![],
            world_map: WorldMap::new(),
        }
    }

    pub fn actor_at_tile(&self, x: i64, y: i64) -> Option<usize> {
        if !self.world_map.is_tile_valid(x, y) {
            return None;
        }

        let player = &self.actors[self.player_index];
        if player.state.position() == (x, y) {
            return None;
        }
        for i in 0..self.actors.len() {
            let actor = &self.actors[i];
            if actor.state.position() == (x, y) {
                return Some(i);
            }
        }
        None
    }

    pub fn is_tile_empty(&self, x: i64, y: i64) -> bool {
        if !self.world_map.is_tile_valid(x, y) {
            return false;
        }
        let player = &self.actors[self.player_index];
        if player.state.position() == (x, y) {
            return false;
        }
        for actor in &self.actors {
            if actor.state.position() == (x, y) {
                return false;
            }
        }

        true
    }
}
