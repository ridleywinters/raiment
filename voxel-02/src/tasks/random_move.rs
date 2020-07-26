use crate::occupation::{Context, Task, TaskStatus};
use rand::Rng;

// Wait 200ms
// Try to move a direction
// Wait 400ms
#[derive(PartialEq)]
enum RandomMoveState {
    Init,
    Wait0(u64),
    Move,
    Wait1 { expiration: u64 },
    Success,
    Fail,
}

pub struct RandomMove {
    state: RandomMoveState,
}

impl RandomMove {
    pub fn new() -> RandomMove {
        RandomMove {
            state: RandomMoveState::Init,
        }
    }
}

impl Task for RandomMove {
    fn update(&mut self, ctx: &mut Context) {
        match self.state {
            RandomMoveState::Init => {
                let mut rng = rand::thread_rng();
                let value = rng.gen_range(200, 600);
                self.state = RandomMoveState::Wait0(ctx.game_time + value);
            }
            RandomMoveState::Wait0(expiration) => {
                if ctx.game_time > expiration {
                    self.state = RandomMoveState::Move
                }
            }

            RandomMoveState::Move => {
                let mut rng = rand::thread_rng();
                let mut dx = 0;
                let mut dy = 0;
                if rng.gen_range(0, 100) >= 50 {
                    dx = if rng.gen_range(0, 100) >= 50 { -1 } else { 1 };
                } else {
                    dy = if rng.gen_range(0, 100) >= 50 { -1 } else { 1 };
                }

                let (ax, ay) = ctx.actor_state.position();
                let (nx, ny) = (ax + dx, ay + dy);

                // TODO: probably should not be setting the position directly as
                // we want actor's to interact with each other based on behaviors
                // that may not be known in this scope.  In other words, abstracting
                // this to an ActorMoveAction might make sense eventually?

                let collision = ctx.other_actors.iter().any(|other| {
                    let (ox, oy) = other.state.position();
                    ox == nx && oy == ny
                });
                self.state = if collision {
                    RandomMoveState::Fail
                } else if ctx.world_map.is_tile_valid(nx, ny) {
                    ctx.actor_state.set_position(nx, ny);
                    RandomMoveState::Wait1 {
                        expiration: ctx.game_time + 200,
                    }
                } else {
                    RandomMoveState::Fail
                }
            }

            RandomMoveState::Wait1 { expiration } => {
                if ctx.game_time > expiration {
                    self.state = RandomMoveState::Success
                }
            }
            RandomMoveState::Success => {}
            RandomMoveState::Fail => {}
        }
    }

    fn status(&self, _: &mut Context) -> TaskStatus {
        match self.state {
            RandomMoveState::Success => TaskStatus::Success,
            RandomMoveState::Fail => TaskStatus::Failure,
            _ => TaskStatus::Active,
        }
    }
}
