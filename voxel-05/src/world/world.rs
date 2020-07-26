use crate::actor::Actor;

use crate::world::world_map::WorldMap;

pub struct Graphics {
    pub sync_id: u64,
    pub node: kiss3d::scene::SceneNode,
}

pub struct WorldEntityState {
    sync_id: u64,

    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub width: u8,
    pub length: u8,
    pub height: u8,

    color: (f32, f32, f32),
}

impl WorldEntityState {
    pub fn sync_id(&self) -> u64 {
        self.sync_id
    }

    pub fn color(&self) -> (f32, f32, f32) {
        self.color
    }

    pub fn set_color(&mut self, color: (f32, f32, f32)) {
        self.sync_id += 1;
        self.color = color;
    }
}

pub struct WorldEntity {
    pub state: WorldEntityState,
    pub graphics: Option<Graphics>,
}

impl WorldEntity {
    fn new(x: i64, y: i64, z: i64, w: i64, l: i64, h: i64) -> Self {
        let state = WorldEntityState {
            sync_id: 0,
            x,
            y,
            z,
            width: w as u8,
            length: l as u8,
            height: h as u8,
            color: (1.0, 1.0, 1.0),
        };
        Self {
            state,
            graphics: None,
        }
    }
}

pub struct WorldEntityList {
    pub entities: Vec<WorldEntity>,
}

impl WorldEntityList {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }
    pub fn add(&mut self, x: i64, y: i64, z: i64, w: i64, l: i64, h: i64, color: (f32, f32, f32)) {
        let mut entity = WorldEntity::new(x, y, z, w, l, h);
        entity.state.set_color(color);
        self.entities.push(entity);
    }
}

pub struct World {
    _seed: usize,
    pub player_index: usize,
    pub actors: Vec<Actor>,
    pub entities: WorldEntityList,
    pub world_map: WorldMap,
}

impl World {
    pub fn new(_rng: &mut rand::rngs::ThreadRng) -> Self {
        Self {
            _seed: 52329,
            player_index: 0,
            actors: vec![],
            entities: WorldEntityList::new(),
            world_map: WorldMap::new(),
        }
    }

    pub fn actor_at_tile(&self, x: i64, y: i64) -> Option<usize> {
        if !self.world_map.is_tile_valid(x, y) {
            return None;
        }

        if self.player_index < self.actors.len() {
            let player = &self.actors[self.player_index];
            if player.state.position() == (x, y) {
                return None;
            }
        }

        for i in 0..self.actors.len() {
            let actor = &self.actors[i];
            if !actor.state.ethereal() && actor.state.position() == (x, y) {
                return Some(i);
            }
        }
        None
    }

    pub fn is_tile_empty(&self, x: i64, y: i64) -> bool {
        x >= 0
            && x < self.world_map.width()
            && y >= 0
            && y < self.world_map.length()
            && self.actor_at_tile(x, y).is_none()
    }
}
