use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, Strategy};
use crate::world::TileKind;
use rand::Rng;

pub struct GrowPlants {}

impl GrowPlants {
    pub fn new() -> Self {
        Self {}
    }
}

impl Occupation for GrowPlants {
    fn name(&self) -> &'static str {
        return "GrowPlants";
    }

    fn init(&self, actor_state: &mut ActorState) {
        actor_state.set_color(0.8, 0.8, 0.8);
    }

    fn update(&self, _: u64) {}

    fn generate_strategy(&self) -> Box<dyn Strategy> {
        Box::new(GrowStrategy::new())
    }
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

#[derive(PartialEq, Clone)]
enum GrowPlanState {
    Init,
    Wait(u64, Box<GrowPlanState>),
    Grow,
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
                if tile.kind == TileKind::Tilled && tile.age() > 200 {
                    ctx.map.set_kind(x, y, TileKind::Plants, true);
                } else if tile.kind == TileKind::Plants && tile.age() > 200_000 {
                    ctx.map.set_kind(x, y, TileKind::Grass, true);
                }
                self.state = Wait(rng.gen_range(300, 3000), Box::new(Grow));
            }
        }
    }
}
