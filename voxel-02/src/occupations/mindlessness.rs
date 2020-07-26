//
// An Occupation defines its Strategies and Plans in the same
// file as these, unlike individual tasks, are generally not
// reused outside the context of that Occupation.
//
use std::collections::VecDeque;

use crate::occupation::{Context, Occupation, PlanStatus, Strategy, Task, TaskStatus};
use crate::tasks;

//
// Occupation
//
pub struct Mindlessness {}

impl Occupation for Mindlessness {
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
        match self.plan.status() {
            PlanStatus::Active => {
                self.plan.update(context);
            }
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

    pub fn update(&mut self, mut ctx: Context) {
        if self.tasks.len() == 0 {
            return;
        }

        let current_task = &mut self.tasks[0];
        current_task.update(&mut ctx);
        match current_task.status(&mut ctx) {
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
    }

    pub fn status(&self) -> PlanStatus {
        if self.tasks.len() > 0 {
            PlanStatus::Active
        } else {
            PlanStatus::Success
        }
    }
}
