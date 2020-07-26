use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, Strategy, Task, TaskStatus};
use rand::Rng;

pub struct Farmer {}

impl Farmer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Occupation for Farmer {
    fn name(&self) -> &'static str {
        return "Farmer";
    }

    fn init(&self, actor_state: &mut ActorState) {
        actor_state.set_color(0.8, 0.8, 0.0);
    }

    fn update(&self, _: u64) {}

    fn generate_strategy(&self) -> Box<dyn Strategy> {
        Box::new(FarmingStrategy::new())
    }
}

struct FarmingStrategy {
    plan: PlotPlan,
}

impl FarmingStrategy {
    pub fn new() -> Self {
        Self {
            plan: PlotPlan {
                plot: (0, 0, 0, 0),
                state: PlotPlanState::ChoosePlot,
            },
        }
    }
}

impl Strategy for FarmingStrategy {
    fn update(&mut self, context: Context) {
        self.plan.update(context);
    }
}

#[derive(PartialEq)]
enum PlotPlanState {
    ChoosePlot,
    LevelTerrain { p: (i64, i64), q: (i64, i64) },
    MoveThenDig(MoveTo, DigTask),
    Dig(DigTask),
    WaitThenLevel(u64),
    TillPlot,
    MoveThenTill(MoveTo, TillTask),
    Till(TillTask),
    Done(u64),
}

struct PlotPlan {
    plot: (i64, i64, i64, i64),
    state: PlotPlanState,
}

impl PlotPlan {
    fn update(&mut self, mut ctx: Context) {
        use PlotPlanState::*;

        match self.state {
            ChoosePlot => {
                println!("Choosing plot");
                let mut rng = rand::thread_rng();
                let width = rng.gen_range(4, 8 + 1);
                let height = rng.gen_range(4, 8 + 1);

                let x0 = rng.gen_range(0, ctx.map.width() - width);
                let y0 = rng.gen_range(0, ctx.map.length() - height);
                let x1 = x0 + width;
                let y1 = y0 + height;

                self.plot = (x0, y0, x1, y1);
                self.state = PlotPlanState::LevelTerrain {
                    p: (x0, y0),
                    q: (x1, y1),
                }
            }

            LevelTerrain { p, q } => {
                // Could move this part to its own stage
                let mut min_z = ctx.map.height(p.0, p.1);
                for y in p.1..q.1 {
                    for x in p.0..q.0 {
                        let z = ctx.map.height(x, y);
                        min_z = min_z.min(z);
                    }
                }

                let min_z = min_z;
                let mut r = None;
                for y in p.1..q.1 {
                    for x in p.0..q.0 {
                        let z = ctx.map.height(x, y);
                        if z > min_z {
                            r = Some((x, y, z - 1));
                        }
                    }
                }

                if let Some((x, y, z)) = r {
                    self.state = MoveThenDig(
                        MoveTo {
                            destination: (x, y),
                        },
                        DigTask {
                            destination: (x, y),
                            height: z as _,
                        },
                    );
                } else {
                    self.state = TillPlot;
                }
            }

            MoveThenDig(ref mut task, dig_task) => match task.update(&mut ctx) {
                TaskStatus::Success => {
                    self.state = Dig(dig_task);
                }
                _ => {}
            },

            Dig(ref mut task) => match task.update(&mut ctx) {
                TaskStatus::Success => {
                    ctx.actor_state.inc_sync_id();
                    self.state = WaitThenLevel(ctx.game_time + 50);
                }
                _ => {}
            },

            WaitThenLevel(expiration) => {
                if ctx.game_time > expiration {
                    self.state = LevelTerrain {
                        p: (self.plot.0, self.plot.1),
                        q: (self.plot.2, self.plot.3),
                    };
                }
            }

            TillPlot => {
                let p = (self.plot.0, self.plot.1);
                let q = (self.plot.2, self.plot.3);

                let mut r = None;
                for y in p.1..q.1 {
                    for x in p.0..q.0 {
                        let tile = ctx.map.tile(x, y);
                        if tile.kind != 2 {
                            r = Some((x, y, 1));
                        }
                    }
                }

                if let Some((x, y, z)) = r {
                    self.state = MoveThenTill(
                        MoveTo {
                            destination: (x, y),
                        },
                        TillTask {
                            destination: (x, y),
                            kind: 2,
                        },
                    )
                } else {
                    self.state = Done(ctx.game_time + 1500);
                }
            }

            MoveThenTill(ref mut task, till_task) => match task.update(&mut ctx) {
                TaskStatus::Success => {
                    self.state = Till(till_task);
                }
                _ => {}
            },

            Till(ref mut task) => match task.update(&mut ctx) {
                TaskStatus::Success => self.state = TillPlot,
                _ => {}
            },

            Done(expiration) => {
                if ctx.game_time > expiration {
                    self.state = ChoosePlot;
                }
            }
        };
    }
}

#[derive(PartialEq)]
struct MoveTo {
    destination: (i64, i64),
}

impl Task for MoveTo {
    fn update(&mut self, ctx: &mut Context) -> TaskStatus {
        let (px, py) = ctx.actor_state.position();
        let (dx, dy) = (self.destination.0 - px, self.destination.1 - py);

        if dx < 0 {
            ctx.actor_state.set_position(px - 1, py);
        } else if dx > 0 {
            ctx.actor_state.set_position(px + 1, py);
        } else if dy < 0 {
            ctx.actor_state.set_position(px, py - 1);
        } else if dy > 0 {
            ctx.actor_state.set_position(px, py + 1);
        } else {
            return TaskStatus::Success;
        }
        TaskStatus::Active
    }
}

#[derive(PartialEq, Copy, Clone)]
struct DigTask {
    destination: (i64, i64),
    height: i64,
}

impl Task for DigTask {
    fn update(&mut self, ctx: &mut Context) -> TaskStatus {
        let (px, py) = ctx.actor_state.position();
        let (dx, dy) = (self.destination.0 - px, self.destination.1 - py);

        if dx != 0 || dy != 0 {
            // Somehow not in the right location!
            println!("Not in the right location to dig!");
            return TaskStatus::Failure;
        }

        if dx == 0 && dy == 0 && ctx.map.height(px, py) as i64 > self.height {
            ctx.map.set_height(px, py, self.height);
        }
        TaskStatus::Success
    }
}

#[derive(PartialEq, Copy, Clone)]
struct TillTask {
    destination: (i64, i64),
    kind: i16,
}

impl Task for TillTask {
    fn update(&mut self, ctx: &mut Context) -> TaskStatus {
        let (px, py) = ctx.actor_state.position();
        let (dx, dy) = (self.destination.0 - px, self.destination.1 - py);

        if dx != 0 || dy != 0 {
            // Somehow not in the right location!
            println!("Not in the right location!");
            return TaskStatus::Failure;
        }

        if dx == 0 && dy == 0 {
            ctx.map.set_kind(px, py, self.kind);
        }
        TaskStatus::Success
    }
}
