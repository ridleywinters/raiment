use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, Strategy, Task, TaskStatus};
use crate::tasks;
use crate::world::{FindPathOptions, TileKind};
use rand::Rng;
use std::collections::HashSet;

/*
    Pseudo-code
    - Move around randomly for a while
    - Choose a starting concrete tile or an edge tile
    - Choose a ending concrete tile or an edge tile
    - Find a path between them
    - Lock those tiles
    - Change each tile to concrete
    - Wait and restart
*/

pub struct RoadBuilder {}

impl RoadBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl Occupation for RoadBuilder {
    fn name(&self) -> &'static str {
        return "Road Builder";
    }

    fn init(&self, actor_state: &mut ActorState) {
        actor_state.set_color(0.55, 0.69, 0.93);
    }

    fn update(&self, _: u64) {}

    fn generate_strategy(&self) -> Box<dyn Strategy> {
        Box::new(RoadStrategy::new())
    }
}

#[derive(PartialEq, Clone)]
enum PlanState {
    Init,
    Wander {
        iterations: u8,
    },
    ChoosePath,
    GotoPath,
    PavePath,

    WaitThen {
        expiration: u64,
        next_state: Box<PlanState>,
    },
}

struct RoadStrategy {
    state: PlanState,
    active_task: Option<Box<dyn Task>>,
    path_key: u64,
    path: Vec<(i64, i64)>,
    move_path: Vec<(i64, i64)>,
    next_move: u64,
}

impl RoadStrategy {
    pub fn new() -> Self {
        Self {
            state: PlanState::Init,
            active_task: None,
            path_key: 0,
            path: Vec::new(),
            move_path: Vec::new(),
            next_move: 0,
        }
    }
}

impl Strategy for RoadStrategy {
    fn update(&mut self, mut ctx: Context) {
        use PlanState::*;

        match self.state {
            Init => {
                self.state = Wander { iterations: 2 };
            }
            Wander { iterations } => {
                if iterations == 0 {
                    self.state = ChoosePath;
                    return;
                }
                if self.active_task.is_none() {
                    self.active_task = Some(Box::new(tasks::RandomMove::new()));
                }

                let task = self.active_task.as_mut().unwrap();
                if task.update(&mut ctx) != TaskStatus::Active {
                    self.active_task = None;
                    self.state = Wander {
                        iterations: iterations - 1,
                    };
                }
            }
            ChoosePath => {
                let mut x0 = ctx.rng.gen_range(0, ctx.map.width());
                let mut x1 = ctx.rng.gen_range(0, ctx.map.width());
                let mut y0 = ctx.rng.gen_range(0, ctx.map.length());
                let mut y1 = ctx.rng.gen_range(0, ctx.map.length());

                if x1 < x0 {
                    std::mem::swap(&mut x0, &mut x1);
                }
                if y1 < y0 {
                    std::mem::swap(&mut y0, &mut y1);
                }

                let width = x1 - x0;
                let length = y1 - y0;

                if width > 6 && length > 6 && width + length > 32 {
                    let mut opts = FindPathOptions::new();
                    opts.add_invalid_tile(TileKind::Plants);
                    opts.add_invalid_tile(TileKind::Tilled);

                    let path = ctx.map.find_path((x0, y0), (x1, y1), Some(opts));
                    if let Some(path) = path {
                        if let Some(key) = ctx.map.try_lock_path(&path) {
                            let pos = ctx.actor_state.position();
                            let start = path[0];
                            if let Some(move_path) = ctx.map.find_path(pos, start, None) {
                                self.move_path = move_path;
                                self.move_path.reverse();

                                self.path_key = key;
                                self.path = path;
                                self.path.reverse();
                                self.state = GotoPath;
                            }
                        }
                    }
                }
            }

            GotoPath => {
                if self.next_move > ctx.game_time {
                    return;
                }

                if self.move_path.len() > 0 {
                    let (x, y) = self.move_path.pop().unwrap();
                    ctx.actor_state.set_position(x, y);
                    let tile = ctx.map.tile(x, y);
                    let speed = match tile.kind {
                        TileKind::Concrete => 20,
                        _ => 100,
                    };
                    self.next_move = ctx.game_time + speed;
                } else {
                    self.state = PavePath;
                }
            }

            PavePath => {
                if self.next_move > ctx.game_time {
                    return;
                }

                if self.path.len() > 0 {
                    let (x, y) = self.path.pop().unwrap();
                    ctx.actor_state.set_position(x, y);

                    let tile = ctx.map.tile(x, y);
                    let mut speed = 100;
                    match tile.kind {
                        TileKind::Concrete => {
                            speed = 20;
                        }
                        _ => {
                            ctx.map.set_kind(x, y, TileKind::Concrete, true);
                        }
                    };
                    self.next_move = ctx.game_time + speed;
                } else {
                    ctx.map.unlock_path(self.path_key);
                    self.path_key = 0;
                    self.state = WaitThen{
                        expiration : ctx.game_time + ctx.rng.gen_range(1_000, 10_000),
                        next_state : Box::new(Init),
                    }
                }
            }

            WaitThen {
                expiration,
                ref next_state,
            } => {
                if ctx.game_time > expiration {
                    self.state = *next_state.clone();
                }
            }
        }
    }
}
