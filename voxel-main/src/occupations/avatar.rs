use crate::actor::ActorState;
use crate::occupation::{Context, Occupation, Strategy};

pub struct Avatar {}

impl Occupation for Avatar {
    fn name(&self) -> &'static str {
        return "Avatar";
    }

    fn init(&self, actor_state: &mut ActorState) {
        actor_state.set_color(0.2, 0.5, 0.8);
    }

    fn update(&self, _: u64) {}

    fn generate_strategy(&self) -> Box<dyn Strategy> {
        Box::new(AvatarStrategy::new())
    }
}

impl Avatar {
    pub fn new() -> Self {
        Self {}
    }
}

//
// Strategy
//

struct AvatarStrategy {}

impl AvatarStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Strategy for AvatarStrategy {
    fn update(&mut self, _ctx: Context) {
        // No-op since the Avatar is, by definition, controlled by
        // something else.
    }
}
