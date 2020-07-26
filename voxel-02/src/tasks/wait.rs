use crate::occupation::{Context, Task, TaskStatus};
use rand::Rng;

pub struct Wait {
    expiration: u64,
    start_time: u64,
    stage: u64,
    prior_color: (f32, f32, f32),
}

impl Wait {
    pub fn new(game_time: u64, min: u64, max: u64) -> Self {
        let mut rng = rand::thread_rng();
        let value = rng.gen_range(min, max);
        Wait {
            expiration: game_time + value,
            start_time: game_time,
            stage: 0,
            prior_color: (0.0, 0.0, 0.0),
        }
    }
}

impl Task for Wait {
    fn update(&mut self, ctx: &mut Context) {
        if self.stage == 0 {
            self.prior_color = ctx.actor_state.color();
            ctx.actor_state.set_color(0.0, 1.0, 0.0);
            self.stage = 1;
        }
        if ctx.game_time > self.expiration {
            let (r, g, b) = self.prior_color;
            ctx.actor_state.set_color(r, g, b);
        } else {
            let d = (self.expiration - self.start_time) as f32;
            let e = (ctx.game_time - self.start_time) as f32;
            let mut a = 1.0 - (e / d).min(1.0).max(0.0);
            a = a.powi(3);
            let b = 1.0 - a;

            let (pr, pg, pb) = self.prior_color;
            let (wr, wg, wb) = (1.0, 0.2, 0.2);

            ctx.actor_state
                .set_color(pr * b + wr * a, pg * b + wg * a, pb * b + wb * a);
        }
    }

    fn status(&self, ctx: &mut Context) -> TaskStatus {
        if ctx.game_time > self.expiration {
            TaskStatus::Success
        } else {
            TaskStatus::Active
        }
    }
}
