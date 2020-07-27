use std::collections::HashMap;

use crate::occupation::{Occupation, Strategy};
use crate::occupations;

pub struct Actor {
    pub state: ActorState,
    pub name: String,

    pub occupation: Box<dyn Occupation>,
    pub strategy: Option<Box<dyn Strategy>>,

    pub shortterm_memory: HashMap<String, std::time::Instant>,

    pub node_sync_id: u64,
    pub node: Option<kiss3d::scene::SceneNode>,
}

impl Actor {
    pub fn new() -> Actor {
        Actor {
            state: ActorState::new(),
            name: String::new(),
            occupation: Box::new(occupations::Avatar::new()),
            strategy: None,

            shortterm_memory: HashMap::new(),

            node_sync_id: 0,
            node: None,
        }
    }
}

// ActorState is split out from the Actor struct to allow for easier borrowing of
// parts of the composite struct.
pub struct ActorState {
    sync_id: u64,
    x: i64,
    y: i64,

    r: f32,
    g: f32,
    b: f32,

    // TODO: the graphics state and physics state should be Option<> or Traits driven, not
    // a bool. Rust can help detect incorrect assumptions using that approach, whereas a
    // runtime bool is more open to programmer error.
    ethereal: bool,
}

impl ActorState {
    fn new() -> Self {
        Self {
            // Start at 1 so any dependent sync_id's that start at 0 are out-of-sync by default
            sync_id: 1,

            x: 0,
            y: 0,

            r: 0.0,
            g: 0.0,
            b: 0.0,

            ethereal: false,
        }
    }

    pub fn sync_id(&self) -> u64 {
        self.sync_id
    }

    pub fn inc_sync_id(&mut self) {
        self.sync_id += 1;
    }

    pub fn position(&self) -> (i64, i64) {
        (self.x, self.y)
    }

    pub fn color(&self) -> (f32, f32, f32) {
        (self.r, self.g, self.b)
    }

    pub fn set_ethereal(&mut self, ethereal: bool) {
        self.ethereal = ethereal;
    }
    pub fn ethereal(&self) -> bool {
        self.ethereal
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
