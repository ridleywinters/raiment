use crate::actor::{Actor, ActorState};
use crate::world::WorldEntityList;
use crate::world::WorldMap;
use rand;

pub trait Occupation {
    fn name(&self) -> &'static str;

    fn generate_strategy(&self) -> Box<dyn Strategy>;

    fn init(&self, _actor_state: &mut ActorState) {}

    // Unclear what this is for yet
    fn update(&self, _game_time: u64) {}
}

pub struct Context<'a> {
    pub game_time: u64,
    pub rng: &'a mut rand::rngs::ThreadRng,
    pub map: &'a mut WorldMap,
    pub entities: &'a mut WorldEntityList,
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

#[derive(PartialEq)]
pub enum TaskStatus {
    Active,
    Success,
    Failure,
}

pub trait Task {
    fn update(&mut self, context: &mut Context) -> TaskStatus;
}

pub enum Status2 {
    Continue,
    Wait(u64),
    Success,
    Failure,
}