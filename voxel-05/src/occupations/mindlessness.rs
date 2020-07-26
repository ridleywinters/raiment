//
// An Occupation defines its Strategies and Plans in the same
// file as these, unlike individual tasks, are generally not
// reused outside the context of that Occupation.
//
use std::collections::VecDeque;

use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, PlanStatus, Strategy, Task, TaskStatus};
use crate::tasks;

//
// Occupation
//
pub struct Mindlessness {}

impl Occupation for Mindlessness {
    fn name(&self) -> &'static str {
        return "Vagrant";
    }

    fn init(&self, actor_state: &mut ActorState) {
        actor_state.set_color(0.3, 0.1, 0.40);
    }

    fn update(&self, _: u64) {}

    fn generate_strategy(&self) -> Box<dyn Strategy> {
        Box::new(MindlessMovements::new())
    }
}

impl Mindlessness {
    pub fn new() -> Self {
        Self {}
    }
}

//
// Strategy
//
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
    fn update(&mut self, context: Context) {
        match self.plan.update(context) {
            PlanStatus::Active => {}
            _ => {
                self.plan = RandomMovements::new();
            }
        }
    }
}

//
// Plan
//
struct RandomMovements {
    tasks: VecDeque<Box<dyn Task>>,
}

impl RandomMovements {
    pub fn new() -> Self {
        let mut tasks = VecDeque::<Box<dyn Task>>::new();
        for _ in 0..8 {
            tasks.push_back(Box::new(tasks::RandomMove::new()));
        }
        Self { tasks }
    }

    pub fn update(&mut self, mut ctx: Context) -> PlanStatus {
        if self.tasks.len() == 0 {
            return PlanStatus::Success;
        }

        let current_task = &mut self.tasks[0];
        match current_task.update(&mut ctx) {
            TaskStatus::Active => {}
            TaskStatus::Success => {
                self.tasks.pop_front();
            }
            TaskStatus::Failure => {
                self.tasks.pop_front();
                self.tasks
                    .push_front(Box::new(tasks::Wait::new(ctx.game_time, 500, 1000)));
            }
        }

        if self.tasks.len() > 0 {
            PlanStatus::Active
        } else {
            PlanStatus::Success
        }
    }
}
