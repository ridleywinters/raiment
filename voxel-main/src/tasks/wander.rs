use rand::seq::SliceRandom;

use crate::occupation::{ Context, Status2 };
use Status2::*;
use WanderTaskState::*;

pub struct WanderTaskBuilder {
    delay_frames: u64,
    duration_frames : u64,
}

impl WanderTaskBuilder {
    pub fn build(self) -> WanderTask {
        use WanderTaskState::*;
        WanderTask {
            state: Init,
            expiration: 0,
            delay_frames: self.delay_frames,
            duration_frames : self.duration_frames,
        }
    }

    pub fn with_delay_frames(mut self, frames: u64) -> WanderTaskBuilder {
        self.delay_frames = frames;
        self
    }

    pub fn with_duration_frames(mut self, frames: u64) -> WanderTaskBuilder {
        self.duration_frames = frames;
        self
    }
}

enum WanderTaskState {
    Init,
    Move,
}

pub struct WanderTask {
    state: WanderTaskState,
    expiration: u64,

    duration_frames : u64,
    delay_frames: u64,
}

impl WanderTask {
    pub fn new() -> WanderTaskBuilder {
        WanderTaskBuilder { 
            delay_frames: 0,
            duration_frames : 3 * 60,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> Status2 {
        

        match self.state {
            Init => {
                self.expiration = ctx.game_time + self.duration_frames;
                self.state = Move;
                Wait(self.delay_frames)
            }
            Move => {
                //
                // Choose a direction
                //
                let movements = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
                let (dx, dy) = movements.choose(ctx.rng).unwrap();

                //
                // Check for a collision and do the move if no collision
                //

                let (ax, ay) = ctx.actor_state.position();
                let (nx, ny) = (ax + dx, ay + dy);

                // TODO: probably should not be setting the position directly as
                // we want actor's to interact with each other based on behaviors
                // that may not be known in this scope.  In other words, abstracting
                // this to an ActorMoveAction might make sense eventually?

                let walkable = ctx.map.is_tile_valid(nx, ny) && ctx.map.tile(nx, ny).is_walkable();
                let collision = !walkable
                    || ctx.other_actors.iter().any(|other| {
                        let (ox, oy) = other.state.position();
                        ox == nx && oy == ny
                    });

                if !collision && ctx.map.is_tile_valid(nx, ny) {
                    ctx.actor_state.set_position(nx, ny);
                }

                //
                // Is the task done?
                //
                match ctx.game_time < self.expiration {
                    true => Wait(self.delay_frames),
                    false => Success,
                }
            }
        }
    }
}
