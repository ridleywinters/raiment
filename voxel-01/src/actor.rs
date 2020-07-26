use std::collections::{HashMap};

use crate::occupation::{Occupation, Strategy};
use crate::occupations;

pub struct Actor {
    pub sync_id: u64,
    pub x: i64,
    pub y: i64,
    pub name: String,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub occupation_name: String,
    pub occupation : Box<dyn Occupation>,
    pub strategy: Option<Box<dyn Strategy>>,

    pub shortterm_memory: HashMap<String, std::time::Instant>,

    pub node_sync_id: u64,
    pub node: Option<kiss3d::scene::SceneNode>,
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.sync_id)
    }
}

impl Actor {
    pub fn new() -> Actor {
        Actor {
            // Start at one so any dependent sync_id's are out-of-sync by default
            sync_id: 1,
            x: 0,
            y: 0,
            name: String::new(),
            r: 0.0,
            g: 0.0,
            b: 0.0,
            occupation_name: String::from("vagabond"),
            occupation: Box::new(occupations::mindlessness::Mindlessness::new()),
            strategy: None,

            shortterm_memory: HashMap::new(),

            node_sync_id: 0,
            node: None,
        }
    }

    pub fn set_position(&mut self, x: i64, y: i64) {
        self.x = x;
        self.y = y;
        self.sync_id += 1;
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.r = r;
        self.g = g;
        self.b = b;
        self.sync_id += 1;
    }
}

