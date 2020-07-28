use rand::Rng;
use std::cell::*;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::world::tile::*;

pub const REGION_SIZE: usize = 64;

#[derive(Copy, Clone)]
pub struct MapRegion {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub length: i64,
}

pub struct FindPathOptions {
    invalid_tilekinds: HashSet<TileKind>,
}

impl FindPathOptions {
    pub fn new() -> Self {
        Self {
            invalid_tilekinds: HashSet::new(),
        }
    }

    pub fn add_invalid_tile(&mut self, kind: TileKind) {
        self.invalid_tilekinds.insert(kind);
    }
}

pub struct WorldRegion {
    pub sync_id: u64,
    pub chunk_sync_ids: HashMap<(i64, i64, i64), u64>,
    pub tiles: [Tile; REGION_SIZE * REGION_SIZE],

    pub region_x: i64, // region = tile / REGION_SIZE
    pub region_y: i64,
}

impl WorldRegion {
    pub fn new(region_x: i64, region_y: i64) -> Self {
        println!("Building heightmap ({}, {})...", region_x, region_y);
        let mut rng = rand::thread_rng();
        let offset_x = rng.gen_range(0.0, 100.0) + region_x as f32 * REGION_SIZE as f32;
        let offset_y = rng.gen_range(0.0, 100.0) + region_y as f32 * REGION_SIZE as f32;
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
            region_x,
            region_y,
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

    pub fn tile_mut(&mut self, x: i64, y: i64) -> &mut Tile {
        let i = (y * self.width() + x) as usize;
        &mut self.tiles[i]
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

    pub fn update(&mut self) {
        // TODO: move call to update_tile_ages here
    }

    pub fn update_tile_ages(&mut self) {
        for i in 0..self.tiles.len() {
            let tile = &mut self.tiles[i];
            tile.inc_age();
        }
    }
}

pub struct WorldMap {
    regions: RefCell<HashMap<(i64, i64), WorldRegion>>,

    // Allow sections of the map to be locked for editing
    // by a particular actor
    lock_id_counter: u64,
    locked_regions: HashMap<u64, MapRegion>,
    locked_paths: HashMap<u64, Vec<(i64, i64)>>,
}

fn coords(x: i64, y: i64) -> (i64, i64, i64, i64) {
    let rx = if x < 0 { x / 64 - 1 } else { x / 64 };
    let ry = if y < 0 { y / 64 - 1 } else { y / 64 };
    let mut tx = x - rx * REGION_SIZE as i64;
    let mut ty = y - ry * REGION_SIZE as i64;

    if rx < 0 {
        tx -= 1;
    }
    if ry < 0 {
        ty -= 1;
    }

    if tx < 0 || tx >= REGION_SIZE as i64 {
        panic!("Invalid tile value: x={}, rx={}, tx={}", x, rx, tx);
    }
    if ty < 0 || ty >= REGION_SIZE as i64 {
        panic!("Invalid tile value: y={}, ry={}, ty={}", y, ry, ty);
    }

    (rx, ry, tx, ty)
}

impl WorldMap {
    pub fn new() -> Self {
        Self {
            regions: RefCell::new(HashMap::new()),
            lock_id_counter: 0,
            locked_regions: HashMap::new(),
            locked_paths: HashMap::new(),
        }
    }

    fn region(&self, x: i64, y: i64) -> Ref<'_, WorldRegion> {
        let rx = if x < 0 { x / 64 - 1 } else { x / 64 };
        let ry = if y < 0 { y / 64 - 1 } else { y / 64 };

        let key = (rx, ry);
        if self.regions.borrow().get(&key).is_none() {
            let region = WorldRegion::new(rx, ry);
            self.regions.borrow_mut().insert(key, region);
        }

        Ref::map(self.regions.borrow(), |m| m.get(&key).unwrap())
    }

    fn region_mut<'a>(&mut self, x: i64, y: i64) -> RefMut<'_, WorldRegion> {
        let rx = if x < 0 { x / 64 - 1 } else { x / 64 };
        let ry = if y < 0 { y / 64 - 1 } else { y / 64 };

        let key = (rx, ry);
        if self.regions.borrow().get(&key).is_none() {
            let region = WorldRegion::new(rx, ry);
            self.regions.borrow_mut().insert(key, region);
        }

        RefMut::map(self.regions.borrow_mut(), |m| m.get_mut(&key).unwrap())
    }

    pub fn height(&self, x: i64, y: i64) -> i32 {
        let (_, _, tx, ty) = coords(x, y);
        self.region(x, y).height(tx, ty)
    }

    pub fn tile(&self, x: i64, y: i64) -> Tile {
        let (_, _, tx, ty) = coords(x, y);
        self.region(x, y).tile(tx, ty)
    }

    pub fn tile_mut(&mut self, x: i64, y: i64) -> RefMut<'_, Tile> {
        let (_, _, tx, ty) = coords(x, y);
        RefMut::map(self.region_mut(x, y), |m| m.tile_mut(tx, ty))
    }

    pub fn tile3(&self, x: i64, y: i64, z: i64) -> TileKind {
        let (_, _, tx, ty) = coords(x, y);
        self.region(x, y).tile3(tx, ty, z)
    }

    pub fn chunk_sync_id(&self, x: i64, y: i64, z: i64) -> u64 {
        let (_, _, tx, ty) = coords(x, y);
        self.region(x, y).chunk_sync_id(tx, ty, z)
    }

    pub fn set_height(&mut self, x: i64, y: i64, z: i64) {
        let (_, _, tx, ty) = coords(x, y);
        self.region_mut(x, y).set_height(tx, ty, z)
    }

    pub fn set_kind(&mut self, x: i64, y: i64, kind: TileKind, reset_age: bool) {
        let (_, _, tx, ty) = coords(x, y);
        self.region_mut(x, y).set_kind(tx, ty, kind, reset_age)
    }

    pub fn is_tile_valid(&self, x: i64, y: i64) -> bool {
        let (_, _, tx, ty) = coords(x, y);
        self.region(x, y).is_tile_valid(tx, ty)
    }

    pub fn update(&mut self) {
        for (_key, region) in self.regions.borrow_mut().iter_mut() {
            region.update();
        }
    }

    pub fn update_tile_ages(&mut self) {
        for (_key, region) in self.regions.borrow_mut().iter_mut() {
            region.update_tile_ages()
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
                self.tile_mut(x, y).set_locked(true);
            }
        }

        let key = self.lock_id_counter;
        self.lock_id_counter += 1;
        self.locked_regions.insert(
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
        let entry = self.locked_regions.get(&key);
        if entry.is_none() {
            return;
        }
        let region = entry.unwrap().clone();

        for y in region.y..region.y + region.length {
            for x in region.x..region.x + region.width {
                self.tile_mut(x, y).set_locked(false);
            }
        }
        self.locked_regions.remove(&key);
    }

    pub fn try_lock_path(&mut self, path: &Vec<(i64, i64)>) -> Option<u64> {
        //
        // Ensure the region is not already locked
        //
        for pair in path {
            let (x, y) = *pair;
            if self.is_tile_locked(x, y) {
                return None;
            }
        }

        //
        // Mark the region as locked
        //
        for pair in path {
            let (x, y) = pair;
            self.tile_mut(*x, *y).set_locked(true);
        }

        let key = self.lock_id_counter;
        self.lock_id_counter += 1;
        self.locked_paths.insert(key, path.clone());
        Some(key)
    }

    pub fn unlock_path(&mut self, key: u64) {
        match self.locked_paths.get(&key) {
            Some(path) => {
                for pair in path.clone() {
                    let (x, y) = pair;
                    self.tile_mut(x, y).set_locked(false);
                }
                self.locked_paths.remove(&key);
            }
            None => {
                panic!("Unexpected key");
            }
        }
    }

    // TODO: is this method needed?
    pub fn is_tile_locked(&self, x: i64, y: i64) -> bool {
        self.tile(x, y).is_locked()
    }

    // TODO: this is a "just get it working" implementation that is undoubtedly
    // *far* less efficient than is theoretically possible
    //
    pub fn find_path(
        &self,
        begin: (i64, i64),
        end: (i64, i64),
        options: Option<FindPathOptions>,
    ) -> Option<Vec<(i64, i64)>> {
        // https://stackoverflow.com/questions/43420605/which-algorithm-from-petgraph-will-find-the-shortest-path-from-a-to-b

        let options = options.unwrap_or(FindPathOptions::new());

        use petgraph::{algo, prelude::*};

        let mut graph = Graph::new();

        let mut dict = HashMap::new();
        let mut tcid = HashMap::new();
        for y in 0..(REGION_SIZE as i64) {
            for x in 0..(REGION_SIZE as i64) {
                let tile = self.tile(x, y);
                if !tile.is_walkable() {
                    continue;
                }
                if options.invalid_tilekinds.contains(&tile.kind) {
                    continue;
                }

                let node = graph.add_node(format!("{},{}", x, y));
                dict.insert((x, y), node);
                tcid.insert(node, (x, y));
            }
        }

        let mut add_edge = |node, tile: &Tile, ex, ey| {
            if let Some(neighbor) = dict.get(&(ex, ey)) {
                let nile = self.tile(ex, ey);
                let mut cost = 5 * (nile.height - tile.height).max(0) as i32;

                // Strongly favor a grid
                if ex % 16 != 3 && ey % 16 != 3 {
                    cost += 10;
                }

                match nile.kind {
                    TileKind::Concrete => {}
                    TileKind::Plants => {
                        cost += 10;
                    }
                    TileKind::Tilled => {
                        cost += 5;
                    }
                    _ => {
                        cost += 1;
                    }
                }
                graph.add_edge(node, *neighbor, 5 + cost);
            }
        };

        for y in 0..(REGION_SIZE as i64) {
            for x in 0..(REGION_SIZE as i64) {
                let entry = dict.get(&(x, y));
                if entry.is_none() {
                    continue;
                }
                let node = *entry.unwrap();
                let tile = self.tile(x, y);

                add_edge(node, &tile, x, y + 1);
                add_edge(node, &tile, x, y - 1);
                add_edge(node, &tile, x + 1, y);
                add_edge(node, &tile, x - 1, y);
            }
        }

        // Path doesn't start or end on a valid tile?
        if !dict.contains_key(&begin) || !dict.contains_key(&end) {
            return None;
        }

        let start = *dict.get(&begin).unwrap();
        let dest = *dict.get(&end).unwrap();
        let path = algo::astar(
            &graph,
            start,           // start
            |n| n == dest,   // is_goal
            |e| *e.weight(), // edge_cost
            |_| 0,           // estimate_cost
        );

        match path {
            Some((_cost, path)) => Some(
                path.iter()
                    .map(|index| *tcid.get(&index).unwrap())
                    .collect::<Vec<(i64, i64)>>(),
            ),
            None => None,
        }
    }
}
