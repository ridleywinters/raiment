use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, Strategy};
use crate::world::TileKind;
use rand::Rng;

pub struct CleanRoads {}

impl CleanRoads {
    pub fn new() -> Self {
        Self {}
    }
}

impl Occupation for CleanRoads {
    fn name(&self) -> &'static str {
        return "CleanRoads";
    }

    fn init(&self, actor_state: &mut ActorState) {
        actor_state.set_color(0.8, 0.8, 0.8);
    }

    fn update(&self, _: u64) {}

    fn generate_strategy(&self) -> Box<dyn Strategy> {
        Box::new(GrowStrategy::new())
    }
}

#[derive(PartialEq, Clone)]
enum GrowPlanState {
    Init,
    Wait(u64, Box<GrowPlanState>),
    Grow,
}

struct GrowStrategy {
    state: GrowPlanState,
}

impl GrowStrategy {
    pub fn new() -> Self {
        Self {
            state: GrowPlanState::Init,
        }
    }
}

impl Strategy for GrowStrategy {
    fn update(&mut self, ctx: Context) {
        use GrowPlanState::*;

        match self.state {
            Init => {
                self.state = Wait(ctx.rng.gen_range(300, 3000), Box::new(Grow));
            }
            Wait(expiration, ref next_state) => {
                if ctx.game_time > expiration {
                    self.state = *next_state.clone();
                }
            }
            Grow => {
                let mut rng = rand::thread_rng();
                let x = rng.gen_range(-150, 150);
                let y = rng.gen_range(-150, 150);
                let tile = ctx.map.tile(x, y);
                if tile.kind == TileKind::Concrete && tile.age() > 10_000 {
                    let mut count = 0;

                    if !ctx.map.is_tile_valid(x - 1, y) ||
                        ctx.map.tile(x - 1, y).kind == TileKind::Concrete
                    {
                        count += 1;
                    }
                    if !ctx.map.is_tile_valid(x + 1, y) ||
                        ctx.map.tile(x + 1, y).kind == TileKind::Concrete
                    {
                        count += 1;
                    }
                    if !ctx.map.is_tile_valid(x, y - 1)
                        || ctx.map.tile(x, y - 1).kind == TileKind::Concrete
                    {
                        count += 1;
                    }
                    if !ctx.map.is_tile_valid(x, y + 1)
                        || ctx.map.tile(x, y + 1).kind == TileKind::Concrete
                    {
                        count += 1;
                    }
                    if count < 2 {
                        println!("Removing concrete at {}, {}", x, y);
                        ctx.map.set_kind(x, y, TileKind::Grass, true);
                    }
                }
                self.state = Wait(rng.gen_range(300, 3000), Box::new(Grow));
            }
        }
    }
}
