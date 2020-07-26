use crate::actor::{Actor, ActorState};
use crate::world::WorldMap;

pub trait Occupation {
    fn name(&self) -> &'static str;

    fn generate_strategy(&self) -> Box<dyn Strategy>;

    fn init(&self, actor_state: &mut ActorState) {}

    // Unclear what this is for yet
    fn update(&self, game_time: u64) {}
}

pub struct Context<'a> {
    pub game_time: u64,
    pub map: &'a mut WorldMap,
    pub actor_state: &'a mut ActorState,
    pub other_actors: &'a [Actor],
}

pub trait Strategy {
    fn update(&mut self, context: Context);
}

pub enum PlanStatus {
    Active,
    Success,
    Failure,
}

pub struct Plan {
    ignored: u64,
}

pub enum TaskStatus {
    Active,
    Success,
    Failure,
}

pub trait Task {
    fn update(&mut self, context: &mut Context) -> TaskStatus;
}
