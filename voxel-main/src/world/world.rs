use rand::seq::SliceRandom;
use rand::Rng;

use crate::actor::Actor;
use crate::occupation::Occupation;
use crate::world::world_map::WorldMap;
use crate::world_entity::*;

pub struct ActorBuilder {
    name: Option<String>,
    is_player: bool,
    is_ethereal: bool,
    position: Option<(i64, i64)>,
    beacon: Option<(i64, i64)>,
}

impl ActorBuilder {
    fn new() -> Self {
        Self {
            name: None,
            is_player: false,
            is_ethereal: false,
            position: None,
            beacon: None,
        }
    }

    pub fn with_name<'a>(&'a mut self, name: &'a str) -> &'a mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_position<'a>(&'a mut self, position: (i64, i64)) -> &'a mut Self {
        self.position = Some(position);
        if self.beacon.is_none() {
            self.beacon = Some(position);
        }
        self
    }

    pub fn with_beacon_point<'a>(&'a mut self, position: (i64, i64)) -> &'a mut Self {
        self.beacon = Some(position);
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

    pub fn build(&self, world: &mut World, f: &dyn Fn() -> Box<dyn Occupation>) {
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
                    let name = names.choose(&mut world.rng);
                    name.unwrap().to_string()
                }
                Some(name) => name.to_string(),
            };

            let p = if let Some(position) = self.position {
                position
            } else {
                (
                    world.rng.gen_range(0, 64) as i64,
                    world.rng.gen_range(0, 64) as i64,
                )
            };
            actor.state.set_position(p.0, p.1);

            if let Some(p) = self.beacon {
                actor.state.set_beacon_point(p);
            } else {
                actor.state.set_beacon_point(actor.state.position());
            }

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
    pub rng: rand::rngs::ThreadRng,
    pub player_index: usize,
    pub actors: Vec<Actor>,
    pub entities: WorldEntityList,
    pub world_map: WorldMap,
}

impl World {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
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

    // "Empty" means it is not blocked by an actor, object, or unwalkable tile
    pub fn is_tile_empty(&self, x: i64, y: i64) -> bool {
        self.world_map.tile(x, y).is_walkable() && self.actor_at_tile(x, y).is_none()
    }
}
