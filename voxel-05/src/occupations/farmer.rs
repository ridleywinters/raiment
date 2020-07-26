use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, Strategy, Task, TaskStatus};
use crate::tasks;
use crate::world::TileKind;
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
                region_key: 0,
                state: PlotPlanState::Init,
            },
        }
    }
}

impl Strategy for FarmingStrategy {
    fn update(&mut self, context: Context) {
        self.plan.update(context);
    }
}

#[derive(PartialEq, Clone)]
enum PlotPlanState {
    Init,
    ChoosePlot {
        considerations: i32,
        best_delta: Option<i64>,
    },
    LevelTerrain,
    MoveThen(MoveTo, Box<PlotPlanState>),
    Dig(DigTask),
    Wait(u64, Box<PlotPlanState>),
    TillPlot,
    Till(TillTask),
    Done(u64, Box<tasks::RandomMove>),
}

struct PlotPlan {
    plot: (i64, i64, i64, i64),
    region_key: u64,
    state: PlotPlanState,
}

impl PlotPlan {
    fn update(&mut self, mut ctx: Context) {
        use PlotPlanState::*;

        match self.state {
            Init => {
                self.state = Wait(
                    ctx.game_time + ctx.rng.gen_range(500, 5_000),
                    Box::new(PlotPlanState::ChoosePlot {
                        considerations: 40,
                        best_delta: None,
                    }),
                );
            }
            ChoosePlot {
                considerations,
                best_delta,
            } => {
                let width = ctx.rng.gen_range(6, 20 + 1);
                let height = ctx.rng.gen_range(6, 20 + 1);

                let x0 = ctx.rng.gen_range(0, ctx.map.width() - width);
                let y0 = ctx.rng.gen_range(0, ctx.map.length() - height);
                let x1 = x0 + width;
                let y1 = y0 + height;

                // Ensure there is a area that is completely grass AND has a border
                // of 1 tile of grass to consider it a valid plot.
                let mut valid = true;
                let mut min_z = ctx.map.height(x0, y0);
                for y in (y0 - 1)..=y1 {
                    if y < 0 || y == ctx.map.length() {
                        continue;
                    }
                    for x in (x0 - 1)..=x1 {
                        if x < 0 || x == ctx.map.width() {
                            continue;
                        }

                        let tile = ctx.map.tile(x, y);
                        if tile.kind != TileKind::Grass {
                            valid = false;
                        }
                        min_z = min_z.min(tile.height as i32);
                    }
                }

                let mut best_delta = best_delta;
                if valid {
                    let min_z = min_z;
                    let mut delta = 0;
                    for y in y0..y1 {
                        for x in x0..x1 {
                            let z = ctx.map.height(x, y);
                            if z > min_z {
                                delta += z - min_z;
                            }
                        }
                    }

                    let mut update = false;
                    match best_delta {
                        Some(best) => {
                            if (delta as i64) < best {
                                update = true;
                            }
                        }
                        None => {
                            update = true;
                        }
                    }

                    if update {
                        best_delta = Some(delta as i64);
                        self.plot = (x0, y0, x1, y1);
                    }
                }

                //
                // Should we...
                // Consider more plots before making a choice?
                // Start with a new set of considerations since no good plots were found?
                // A good plot was found but someone else claimed it?
                //
                if considerations == 0 {
                    if best_delta.is_none() {
                        self.state = ChoosePlot {
                            considerations: 4,
                            best_delta: None,
                        }
                    } else {
                        let (x0, y0, x1, y1) = self.plot;
                        if let Some(key) = ctx.map.try_lock_region(x0, y0, x1, y1) {
                            self.region_key = key;
                            self.state = PlotPlanState::Wait(
                                ctx.game_time + ctx.rng.gen_range(100, 300),
                                Box::new(PlotPlanState::LevelTerrain),
                            );
                        } else {
                            self.state = ChoosePlot {
                                considerations: 4,
                                best_delta: None,
                            }
                        }
                    }
                } else {
                    self.state = ChoosePlot {
                        considerations: considerations - 1,
                        best_delta: best_delta,
                    };
                }
            }

            LevelTerrain => {
                let (x0, y0, x1, y1) = self.plot;

                // Could move this part to its own stage
                let mut min_z = ctx.map.height(x0, y0);
                for y in y0..y1 {
                    for x in x0..x1 {
                        let z = ctx.map.height(x, y);
                        min_z = min_z.min(z);
                    }
                }

                let min_z = min_z;
                let mut r = None;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let z = ctx.map.height(x, y);
                        if z > min_z {
                            r = Some((x, y, z - 1));
                        }
                    }
                }

                if let Some((x, y, z)) = r {
                    self.state = MoveThen(
                        MoveTo::new(x, y),
                        Box::new(Dig(DigTask {
                            destination: (x, y),
                            height: z as _,
                        })),
                    );
                } else {
                    self.state = TillPlot;
                }
            }

            MoveThen(ref mut task, ref next_state) => match task.update(&mut ctx) {
                TaskStatus::Success => {
                    self.state = *next_state.clone();
                }
                _ => {}
            },

            Dig(ref mut task) => match task.update(&mut ctx) {
                TaskStatus::Success => {
                    ctx.actor_state.inc_sync_id();
                    self.state = Wait(ctx.game_time + 50, Box::new(PlotPlanState::LevelTerrain));
                }
                _ => {}
            },

            Wait(expiration, ref next_state) => {
                if ctx.game_time > expiration {
                    self.state = *next_state.clone();
                }
            }

            TillPlot => {
                let (x0, y0, x1, y1) = self.plot;

                let mut r = None;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let tile = ctx.map.tile(x, y);
                        if tile.kind != TileKind::Tilled && tile.kind != TileKind::Plants {
                            r = Some((x, y, 1));
                        }
                    }
                }

                if let Some((x, y, _z)) = r {
                    self.state = MoveThen(
                        MoveTo::new(x, y),
                        Box::new(Till(TillTask {
                            destination: (x, y),
                            kind: TileKind::Tilled,
                        })),
                    );
                } else {
                    ctx.map.unlock_region(self.region_key);
                    self.region_key = 0;

                    self.state = Done(
                        ctx.game_time + ctx.rng.gen_range(5000, 10_000),
                        Box::new(tasks::RandomMove::new()),
                    );
                }
            }
            Till(ref mut task) => match task.update(&mut ctx) {
                TaskStatus::Success => self.state = TillPlot,
                _ => {}
            },

            Done(expiration, ref mut task) => {
                if ctx.game_time > expiration {
                    self.state = ChoosePlot {
                        considerations: 10,
                        best_delta: None,
                    }
                } else {
                    if task.update(&mut ctx) != TaskStatus::Active {
                        task.reset();
                    }
                }
            }
        };
    }
}

#[derive(PartialEq, Clone, Copy)]
struct MoveTo {
    destination: (i64, i64),
    expiration: u64,
}

impl MoveTo {
    fn new(x: i64, y: i64) -> Self {
        Self {
            destination: (x, y),
            expiration: 0,
        }
    }
}

impl Task for MoveTo {
    fn update(&mut self, ctx: &mut Context) -> TaskStatus {
        if ctx.game_time < self.expiration {
            return TaskStatus::Active;
        }

        self.expiration = ctx.game_time + 50;

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
    kind: TileKind,
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
            ctx.map.set_kind(px, py, self.kind, true);
        }
        TaskStatus::Success
    }
}
