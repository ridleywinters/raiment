use rand::Rng;
use std::collections::HashMap;

use crate::world::tile::*;

pub const REGION_SIZE: usize = 64;

#[derive(Copy, Clone)]
pub struct MapRegion {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub length: i64,
}

pub struct WorldMap {
    pub sync_id: u64,
    pub chunk_sync_ids: HashMap<(i64, i64, i64), u64>,
    pub tiles: [Tile; REGION_SIZE * REGION_SIZE],

    // Allow sections of the map to be locked for editing
    // by a particular actor
    region_lock_counter: u64,
    region_locks: HashMap<u64, MapRegion>,
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
            [Tile::new(); REGION_SIZE * REGION_SIZE];
        for y in 0..REGION_SIZE {
            for x in 0..REGION_SIZE {
                let a = offset_x
                    + (x as f32 * scale_x) * std::f32::consts::PI / (REGION_SIZE as f32 / 2.0);
                let b = offset_y
                    + (y as f32 * scale_y) * std::f32::consts::PI / (REGION_SIZE as f32 / 4.0);
                let z = 2.0 * ((a.sin() + 0.5) + (b.cos() + 0.5));
                let i = y * REGION_SIZE + x;
                heightmap[i].kind = TileKind::Grass;
                heightmap[i].height = (z as i16).max(1);
            }
        }

        Self {
            sync_id: 1,
            chunk_sync_ids: HashMap::new(),
            tiles: heightmap,

            region_lock_counter: 0,
            region_locks: HashMap::new(),
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

    pub fn tile3(&self, x: i64, y: i64, z: i64) -> TileKind {
        let i = (y * self.width() + x) as usize;
        let tile = self.tiles[i];

        if z < 0 || (tile.height as i64) < z {
            TileKind::Empty
        } else {
            tile.kind
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
            Some(&value) => {
                self.chunk_sync_ids.insert(chunk_id, value + 1);
            }
            _ => {
                self.chunk_sync_ids.insert(chunk_id, 1);
            }
        }
        self.sync_id += 1;
    }

    pub fn set_height(&mut self, x: i64, y: i64, z: i64) {
        self.update_chunk_sync_id(x, y, 0);
        let i = (y * self.width() + x) as usize;
        self.tiles[i].height = z as i16;
    }

    pub fn set_kind(&mut self, x: i64, y: i64, kind: TileKind, reset_age: bool) {
        self.update_chunk_sync_id(x, y, 0);
        let i = (y * self.width() + x) as usize;
        self.tiles[i].kind = kind;

        if reset_age {
            self.tiles[i].set_age(0);
        }
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

    pub fn update() {
        // TODO: move call to update_tile_ages here
    }

    pub fn update_tile_ages(&mut self) {
        for i in 0..self.tiles.len() {
            let tile = &mut self.tiles[i];
            tile.inc_age();
        }
    }

    /// Locking regions is intended to allow Actors to "reserve" an area of the map
    /// so they are not interfered with in subsequent actions. For example, a Farmer
    /// can lock the region they are about to create a farm on.  This allows the farm
    /// area to be created over time without another Actor trying to do something else
    /// with that region (build a house, make a road, etc.).
    ///
    /// Given the nature of the game, it should be assumed these locks *can* be violated,
    /// either intentionally or incidentally. Other Actors are assumed to "willfully"
    /// respect locks. In coding terms, this  means Actors who have a lock should still
    /// have logic for when the lock has been violated; the lock is mostly to "encourage"
    /// a particular behavior.
    ///
    pub fn try_lock_region(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) -> Option<u64> {
        //
        // Ensure the region is not already locked
        //
        for y in y0..y1 {
            for x in x0..x1 {
                if self.is_tile_locked(x, y) {
                    return None;
                }
            }
        }

        //
        // Mark the region as locked
        //
        for y in y0..y1 {
            for x in x0..x1 {
                let i = (y * self.width() + x) as usize;
                self.tiles[i].set_locked(true);
            }
        }

        let key = self.region_lock_counter;
        self.region_lock_counter += 1;
        self.region_locks.insert(
            key,
            MapRegion {
                x: x0,
                y: y0,
                width: x1 - x0,
                length: y1 - y0,
            },
        );
        Some(key)
    }
    pub fn unlock_region(&mut self, key: u64) {
        let entry = self.region_locks.get(&key);
        if entry.is_none() {
            return;
        }
        let region = entry.unwrap();

        for y in region.y..region.y + region.length {
            for x in region.x..region.x + region.width {
                let i = (y * self.width() + x) as usize;
                self.tiles[i].set_locked(false);
            }
        }
        self.region_locks.remove(&key);
    }

    // TODO: is this method needed?
    pub fn is_tile_locked(&self, x: i64, y: i64) -> bool {
        self.tile(x, y).is_locked()
    }
}
