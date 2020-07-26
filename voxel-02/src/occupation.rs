use crate::actor::{Actor, ActorState};
use crate::world::WorldMap;

pub trait Occupation {
    // Unclear what this is for yet
    fn update(&self, game_time: u64);

    fn generate_strategy(&self) -> Box<dyn Strategy>;
}

pub struct Context<'a> {
    pub game_time: u64,
    pub world_map: &'a WorldMap,
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
    fn update(&mut self, context: &mut Context);
    fn status(&self, context: &mut Context) -> TaskStatus;
}
