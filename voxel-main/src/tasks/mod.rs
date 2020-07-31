use crate::occupation::Status2;

pub fn task2_wrapper(mut wait: u64, status: Status2) -> u64 {
    if wait > 0 {
        wait -= 1;
    } else {
        match status {
            Status2::Wait(frames) => {
                wait = frames;
            }
            _ => {}
        };
    }
    wait
}

mod wait;
pub use wait::Wait;

pub mod random_move;
pub use random_move::RandomMove;

mod wander;
pub use wander::WanderTask;

mod move_to;
pub use move_to::MoveToTask;

mod locate_tile;
pub use locate_tile::LocateTileTask;

mod change_tile;
pub use change_tile::{change_tile, ChangeTileTask};
