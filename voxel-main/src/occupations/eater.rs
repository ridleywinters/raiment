use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, Strategy, Task, TaskStatus, Status2};
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

enum EaterState {
    Wander(tasks::WanderTask),
}

struct EaterPlan {
    state: EaterState,
}

impl EaterPlan {
    fn new() -> Self {
        Self {
            state: EaterPlan::default_wander(),
        }
    }
    fn default_wander() -> EaterState {
        EaterState::Wander(tasks::WanderTask::new()
            .with_delay_frames(30)
            .with_duration_frames(2 * 60)
            .build())
    }

    fn update(&mut self, mut ctx: Context) -> Status2 {
        use EaterState::*;
        use Status2::*;

        match self.state {
            Wander(ref mut task) => match task.update(&mut ctx) {
                Status2::Success => {
                    self.state = EaterPlan::default_wander();
                    Continue
                }
                value => value,
            },
        }
    }
}

