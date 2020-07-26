use rand::Rng;
use std::collections::VecDeque;

use crate::occupation::{Occupation, PlanStatus, Strategy, TaskStatus};

use crate::actor::Actor;
use crate::world::World;

pub struct StepInput<'a> {
    game_time: u64,
    world: &'a World,
    actor: &'a mut Actor,
}

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

// Occupation
pub struct Mindlessness {}

impl Occupation for Mindlessness {
    fn update(&self, game_time: u64) {}

    fn generate_strategy(&self) -> Box<dyn Strategy> {
        Box::new(MindlessMovements::new())
    }
}

impl Mindlessness {
    pub fn new() -> Self {
        Self {}
    }
}

// Strategy
struct MindlessMovements {
    plan: RandomMovements,
}

impl MindlessMovements {
    pub fn new() -> Self {
        Self {
            plan: RandomMovements::new(),
        }
    }
}

impl Strategy for MindlessMovements {
    fn update(&mut self, game_time: u64) {
        match self.plan.status() {
            PlanStatus::Active => {
                self.plan.update(game_time);
            }
            _ => {
                self.plan = RandomMovements::new();
            }
        }
    }
}

// Plan
struct RandomMovements {
    tasks: VecDeque<RandomMove>,
}

impl RandomMovements {
    pub fn new() -> Self {
        let mut tasks = VecDeque::new();
        for _ in 0..8 {
            tasks.push_back(RandomMove::new());
        }
        Self { tasks }
    }

    pub fn update(&mut self, game_time: u64) -> PlanStatus {
        if self.tasks.len() == 0 {
            return PlanStatus::Success;
        }
        let current_task = &mut self.tasks[0];
        current_task.update(game_time);
        match current_task.status() {
            TaskStatus::Active => {}
            TaskStatus::Success => {
                self.tasks.pop_front();
            }
            TaskStatus::Failure => {
                // count retries
                // current_tasks.reset()
                // self.tasks.push_front(Wait::new(750))
            }
        }

        PlanStatus::Active
    }

    pub fn status(&self) -> PlanStatus {
        if self.tasks.len() > 0 {
            PlanStatus::Active
        } else {
            PlanStatus::Success
        }
    }
}

struct RandomMove {
    state: RandomMoveState,
}

impl RandomMove {
    pub fn new() -> RandomMove {
        RandomMove {
            state: RandomMoveState::Init,
        }
    }

    pub fn update(&mut self, game_time: u64) {
        match self.state {
            RandomMoveState::Init => {
                let mut rng = rand::thread_rng();
                let value = rng.gen_range(200, 600);
                self.state = RandomMoveState::Wait0(game_time + value);
            }

            RandomMoveState::Wait0(expiration) => {
                if game_time > expiration {
                    self.state = RandomMoveState::Move
                }
            }

            RandomMoveState::Move => {
                /*let mut rng = rand::thread_rng();
                let mut dx = 0;
                let mut dy = 0;
                if rng.gen_range(0, 100) >= 50 {
                    dx = if rng.gen_range(0, 100) >= 50 { -1 } else { 1 };
                } else {
                    dy = if rng.gen_range(0, 100) >= 50 { -1 } else { 1 };
                }
                let nx = state.actor.x + dx;
                let ny = state.actor.y + dy;

                if state.world.is_tile_empty(nx, ny) {
                    state.actor.set_position(nx, ny);
                    self.state = RandomMoveState::Success;
                } else {
                    self.state = RandomMoveState::Fail;
                }*/
                println!("TODO: Random movement");
                self.state = RandomMoveState::Success;
            }

            RandomMoveState::Wait1 { expiration } => {
                if game_time > expiration {
                    self.state = RandomMoveState::Success
                }
            }
            RandomMoveState::Success => {}
            RandomMoveState::Fail => {}
        }
    }

    pub fn status(&self) -> TaskStatus {
        match self.state {            
            RandomMoveState::Success => TaskStatus::Success,
            RandomMoveState::Fail => TaskStatus::Failure,
            _ => TaskStatus::Active,
        }
    }
}
