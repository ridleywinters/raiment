use rand::seq::SliceRandom;
use rand::Rng;

use crate::actor::Actor;
use crate::occupation::Occupation;
use crate::world::world_map::{WorldMap, REGION_SIZE};

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

pub struct ActorBuilder {
    name: Option<String>,
    is_player: bool,
    is_ethereal: bool,
}

impl ActorBuilder {
    fn new() -> Self {
        Self {
            name: None,
            is_player: false,
            is_ethereal: false,
        }
    }

    pub fn with_name<'a>(&'a mut self, name: &'a str) -> &'a mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_player(&mut self, is_player: bool) -> &mut Self {
        self.is_player = is_player;
        self
    }

    pub fn with_ethereal(&mut self, is_ethereal: bool) -> &mut Self {
        self.is_ethereal = is_ethereal;
        self
    }

    pub fn build(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        world: &mut World,
        f: &dyn Fn() -> Box<dyn Occupation>,
    ) {
        let names = vec![
            "Raether", "Telenor", "Sentor", "Baaren", "Celinac", "Coplin", "Boran", "Ilia",
            "Kelis", "Elli", "Len", "Bilric", "Rownal", "Cal", "Wern", "Lendole", "Ilabin",
            "Revor", "Edien", "Dien", "Cien", "Aniken", "Anker", "Matken", "Isotel", "Isse",
            "Lince",
        ];
        for _ in 0..1 {
            let mut actor = Actor::new();

            actor.name = match &self.name {
                None => {
                    let name = names.choose(rng);
                    name.unwrap().to_string()
                }
                Some(name) => name.to_string(),
            };

            actor.state.set_position(
                rng.gen_range(0, REGION_SIZE) as i64,
                rng.gen_range(0, REGION_SIZE) as i64,
            );

            actor.occupation = f();
            actor.occupation.init(&mut actor.state);

            if !self.is_ethereal {
                let (x, y) = actor.state.position();
                if !world.is_tile_empty(x, y) {
                    return;
                }
            } else {
                actor.state.set_ethereal(true);
            }

            println!("Adding {}, the {}", actor.name, actor.occupation.name());

            //
            // Add the Actor
            //
            if self.is_player {
                world.player_index = world.actors.len();
            }
            world.actors.push(actor);
        }
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

    pub fn build_actor(&mut self) -> ActorBuilder {
        ActorBuilder::new()
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
            && self.world_map.tile(x, y).is_walkable()
            && self.actor_at_tile(x, y).is_none()
    }
}
