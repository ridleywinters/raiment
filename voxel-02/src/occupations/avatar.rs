
use crate::occupation::{Context, Occupation, Strategy };

pub struct Avatar {}

impl Occupation for Avatar {
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
struct AvatarStrategy {
}

impl AvatarStrategy {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Strategy for AvatarStrategy {
    fn update(&mut self, context: Context) {
        // No-op since the Avatar is, by definition, controlled by
        // something else.
    }
}
