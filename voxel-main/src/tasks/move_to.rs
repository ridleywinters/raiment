use rand::seq::SliceRandom;

use crate::occupation::{Context, Status2};
use crate::world::TileKind;
use MoveToTaskState::*;
use Status2::*;

pub struct MoveToTaskBuilder {
    destination: (i64, i64),
    move_delay_frames: u64,
}

impl MoveToTaskBuilder {
    pub fn build(self) -> MoveToTask {
        use MoveToTaskState::*;
        MoveToTask {
            state: FindPath,
            destination: self.destination,
            move_delay_frames: self.move_delay_frames,
        }
    }

    // How many frames to delay between each random movement
    pub fn with_move_delay_frames(mut self, frames: u64) -> MoveToTaskBuilder {
        self.move_delay_frames = frames;
        self
    }
}

#[derive(PartialEq, Clone)]
enum MoveToTaskState {
    FindPath,
    FollowPath(Vec<(i64, i64)>),
}

#[derive(PartialEq, Clone)]
pub struct MoveToTask {
    state: MoveToTaskState,
    pub destination: (i64, i64),
    move_delay_frames: u64,
}

impl MoveToTask {
    pub fn new_with_destination(destination: (i64, i64)) -> MoveToTaskBuilder {
        MoveToTaskBuilder {
            destination,
            move_delay_frames: 4,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> Status2 {
        match self.state {
            FindPath => {
                let p = ctx.actor_state.position();
                let q = self.destination;

                if p == q {
                    Success
                } else if let Some(mut path) = ctx.map.find_path(p, q, None) {
                    // Reverse the path so we can pop() off the vector
                    path.reverse();
                    self.state = FollowPath(path);
                    Wait(10 * self.move_delay_frames)
                } else {
                    println!("Failed to find path from {:?} {:?}", p, q);
                    Failure
                }
            }
            FollowPath(ref mut path) => {
                if !path.is_empty() {
                    let (x, y) = path.pop().unwrap();

                    // TODO: check if it's a valid tile. If it's not,
                    // Wander for a bit, then FindPath again from the current location.
                    ctx.actor_state.set_position(x, y);
                    let tile = ctx.map.tile(x, y);

                    // TODO: make this more generic. This is just proof-of-concept
                    // for "walkability" of tile types
                    let speed = match tile.kind {
                        TileKind::Concrete => 2,
                        _ => 1,
                    };
                    Wait(self.move_delay_frames / speed)
                } else {
                    Success
                }
            }
        }
    }
}
