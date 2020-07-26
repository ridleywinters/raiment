use crate::actor::Actor;
use rand::Rng;
use std::collections::HashMap;

pub const REGION_SIZE: usize = 64;

#[derive(Copy, Clone)]
pub struct Tile {
    pub kind: i16,
    pub height: i16,
}

pub struct WorldMap {
    pub sync_id: u64,
    pub chunk_sync_ids : HashMap<(i64, i64, i64), u64>,
    pub tiles: [Tile; REGION_SIZE * REGION_SIZE],
}

impl WorldMap {
    pub fn new() -> Self {
        println!("Building heightmap...");
        let mut rng = rand::thread_rng();
        let offset_x = rng.gen_range(0.0, 100.0);
        let offset_y = rng.gen_range(0.0, 100.0);
        let scale_x = rng.gen_range(0.5, 1.5);
        let scale_y = rng.gen_range(0.5, 1.5);
        

        let mut heightmap: [Tile; REGION_SIZE * REGION_SIZE] =
            [Tile { kind: 0, height: 1 }; REGION_SIZE * REGION_SIZE];
        for y in 0..REGION_SIZE {
            for x in 0..REGION_SIZE {
                let a = offset_x
                    + (x as f32 * scale_x) * std::f32::consts::PI / (REGION_SIZE as f32 / 2.0);
                let b = offset_y
                    + (y as f32 * scale_y) * std::f32::consts::PI / (REGION_SIZE as f32 / 4.0);
                let z = 2.0 * ((a.sin() + 0.5) + (b.cos() + 0.5));
                let i = y * REGION_SIZE + x;
                heightmap[i] = Tile {
                    kind: 1,
                    height: (z as i16).max(1),
                };
            }
        }

        Self {
            sync_id: 1,
            chunk_sync_ids : HashMap::new(),
            tiles: heightmap,
        }
    }

    pub fn width(&self) -> i64 {
        REGION_SIZE as i64
    }

    pub fn length(&self) -> i64 {
        REGION_SIZE as i64
    }

    pub fn tile(&self, x: i64, y: i64) -> Tile {
        let i = (y * self.width() + x) as usize;
        self.tiles[i]
    }

    pub fn tile3(&self, x: i64, y: i64, z: i64) -> u8 {
        let i = (y * self.width() + x) as usize;
        let tile = self.tiles[i];

        if z < 0 || (tile.height as i64) < z {
            0
        } else {
            tile.kind as u8
        }
    }

    pub fn height(&self, x: i64, y: i64) -> i32 {
        let i = (y * self.width() + x) as usize;
        self.tiles[i].height as i32
    }

    pub fn chunk_sync_id(&self, x: i64, y: i64, z: i64) -> u64 {
        let chunk_id = (x / 32, y / 32, z / 32);
        match self.chunk_sync_ids.get(&chunk_id) {
            Some(&value) => value,
            _ => 1,
        }
    }

    fn update_chunk_sync_id(&mut self, x: i64, y: i64, z: i64) {
        let chunk_id = (x / 32, y / 32, z / 32);
        match self.chunk_sync_ids.get(&chunk_id) {
            Some(&value) => { self.chunk_sync_ids.insert(chunk_id, value +1); },
            _ => { self.chunk_sync_ids.insert(chunk_id, 1); },
        }
        self.sync_id += 1;
    }

    pub fn set_height(&mut self, x: i64, y: i64, z: i64) {
        self.update_chunk_sync_id(x,y,0);        
        let i = (y * self.width() + x) as usize;
        self.tiles[i].height = z as i16;
    }

    pub fn set_kind(&mut self, x: i64, y: i64, kind: i16) {
        self.update_chunk_sync_id(x,y,0);
        let i = (y * self.width() + x) as usize;
        self.tiles[i].kind = kind;
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
