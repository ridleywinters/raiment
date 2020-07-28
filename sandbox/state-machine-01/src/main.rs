enum Status {
    Continue,
    Wait(u64),
    Success,
    Failure,
}

trait Task {
    fn update(&mut self) -> Status;
}

struct GeneratePosition {
    out_position : (i64, i64),
}

impl GeneratePosition {
    pub fn new() -> Self {
        Self{
            out_position : (1, 2),
        }
    }
}

struct MoveTo {
    position : (i64, i64),
}

impl MoveTo {
    pub fn new() -> Self {
        Self{
            position : (0,0),
        }
    }
}

struct LocateTile {
    position : (i64, i64)
}

struct LocateTileBuilder {
    Til
}

impl Task for LocateTile {

    fn update(&mut self) -> Status {
        self.position = (1, 2);
        Status::Success
    }
}

enum FindFoodState {
    Init,

    LocateFood(LocateTask),
    MoveToFood,
    MoveToEatingLocation,
    EatFood,
    FoodComa,

    Wander,
    Done,
}

struct FindFoodPlan {

}

impl FindFoodPlan {
    
    fn update(&self) {
        use FindFoodState::*;

        match self.state {
            Init => {
                self.state = LocateFood(tasks::locate_tile()
                    .with_kind(TileKind::Plants)
                    .new())
                Continue
            }
            LocateFood(task) => {
                match task.update() {
                    Success => {
                        self.state = MoveToFood(tasks::move_to()
                            .with_position(task.position())
                            .new())               
                        Continue
                    }                    
                    value => { value }
                }
            }
        }
    }
}



fn main() {

    println!("Done!");
}
