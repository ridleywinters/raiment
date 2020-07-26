use crate::world::World;

pub trait Occupation{
    // Unclear what this is for yet
    fn update(&self, game_time : u64);

    fn generate_strategy(&self) -> Box<dyn Strategy>;
}

pub trait Strategy {
    fn update(&mut self,  game_time : u64);
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

pub struct Task {
    ignored: u64,
}
