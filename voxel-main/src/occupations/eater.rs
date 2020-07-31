use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, Status2, Strategy, Task, TaskStatus};
use crate::tasks;
use crate::world::TileKind;
use rand::seq::SliceRandom;
use rand::Rng;

pub struct Eater {}

impl Eater {
    pub fn new() -> Self {
        Self {}
    }
}

impl Occupation for Eater {
    fn name(&self) -> &'static str {
        return "Eater";
    }

    fn init(&self, actor_state: &mut ActorState) {
        actor_state.set_color(0.8, 0.8, 0.8);
    }

    fn update(&self, _: u64) {}

    fn generate_strategy(&self) -> Box<dyn Strategy> {
        Box::new(EaterStrategy::new())
    }
}

struct EaterStrategy {
    plan: EaterPlan,
    scaffold_wait: u64,
}

impl EaterStrategy {
    pub fn new() -> Self {
        Self {
            plan: EaterPlan::new(),
            scaffold_wait: 0,
        }
    }
}

impl Strategy for EaterStrategy {
    fn update(&mut self, ctx: Context) {
        use Status2::*;

        if self.scaffold_wait > 0 {
            self.scaffold_wait -= 1;
            return;
        }

        match self.plan.update(ctx) {
            Wait(frames) => {
                self.scaffold_wait = frames;
            }
            _ => {}
        }
    }
}

// Wander
// LocateFood
// MoveToFood
// EatFood
// Wander
//
enum EaterState {
    Wander(tasks::WanderTask),
    LocateFood(tasks::LocateTileTask),
    Move(tasks::MoveToTask),
    EatFood(tasks::ChangeTileTask),
}

struct EaterPlan {
    state: EaterState,
}

impl EaterPlan {
    fn new() -> Self {
        Self {
            state: EaterPlan::wander(),
        }
    }
    fn wander() -> EaterState {
        EaterState::Wander(
            tasks::WanderTask::new()
                .with_delay_frames(30)
                .with_duration_frames(2 * 6)
                .build(),
        )
    }

    fn update(&mut self, mut ctx: Context) -> Status2 {
        use EaterState::*;
        use Status2::*;

        match self.state {
            Wander(ref mut task) => match task.update(&mut ctx) {
                Success => {
                    self.state = LocateFood(
                        tasks::LocateTileTask::new(TileKind::Plants)
                            .with_attempts(10)
                            .build(),
                    );
                    Continue
                }
                value => value,
            },
            LocateFood(ref mut task) => match task.update(&mut ctx) {
                Success => {
                    self.state =
                        Move(tasks::MoveToTask::new_with_destination(task.destination).build());
                    Continue
                }
                Failure => {
                    self.state = EaterPlan::wander();
                    Wait(10)
                }
                value => value,
            },

            Move(ref mut task) => match task.update(&mut ctx) {
                Success => {
                    self.state = EatFood(
                        tasks::change_tile(TileKind::Tilled)
                            .with_src(Some(TileKind::Plants))
                            .build(),
                    );
                    Continue
                }
                Failure => {
                    self.state = EaterPlan::wander();
                    Continue
                }
                value => value,
            },

            EatFood(ref mut task) => match task.update(&mut ctx) {
                Success => {
                    self.state = EaterPlan::wander();
                    Wait(20)
                }
                Failure => {
                    self.state = EaterPlan::wander();
                    Wait(5)
                }
                value => value,
            },
        }
    }
}
