use std::collections::VecDeque;

enum Status {
    Active(u64),
    Success,
    Failure,
}

trait Strategy {
    fn update(&mut self) -> Status;
}

trait Task {
    fn update(&mut self) -> Status;
}

enum MoveState {
    Init,
    Move(u8),
    Done,
}

struct Move {
    state: MoveState,
}

impl Move {
    fn new() -> Self {
        Self {
            state: MoveState::Init,
        }
    }
    fn new_task() -> Box<dyn Task> {
        Box::new(Self::new())
    }
}

impl Task for Move {
    fn update(&mut self) -> Status {
        use MoveState::*;
        match self.state {
            Init => {
                self.state = Move(3);
                Status::Active(1)
            }
            Move(ref mut count) => {
                println!("Move {}", count);
                *count -= 1;
                if *count == 0 {
                    self.state = Done;
                }
                Status::Active(0)
            }
            Done => {
                println!("Done moving");
                Status::Success
            }
        }
    }
}

// Think for a bit, then dig, then think for a bit
enum DigState {
    Init,
    Wait0,
    Dig { task: Box<dyn Task> },
    Done,
}

struct Dig {
    state: DigState,
}
impl Dig {
    fn new() -> Self {
        Self {
            state: DigState::Init,
        }
    }
    fn new_task() -> Box<dyn Task> {
        Box::new(Self::new())
    }
}

fn transition<T>(delay: u64, state: T) -> (Status, Option<T>) {
    (Status::Active(delay), Some(state))
}

fn wait<T>(delay: u64) -> (Status, Option<T>) {
    (Status::Active(delay), None)
}
fn success<T>() -> (Status, Option<T>) {
    (Status::Success, None)
}

fn run_task<T>(task: &mut Box<dyn Task>, success_state: T, fail_state: T) -> (Status, Option<T>) {
    match task.update() {
        Status::Active(n) => wait(n),
        Status::Success => transition(0, success_state),
        Status::Failure => transition(0, fail_state),
    }
}

fn state_update<T>(s: &mut T, (status, state): (Status, Option<T>)) -> Status {
    if state.is_some() {
        *s = state.unwrap();
    }
    status
}

impl Task for Dig {
    fn update(&mut self) -> Status {
        use DigState::*;
        let pair = match self.state {
            Init => {
                println!("Init");
                transition(2, Wait0)
            }
            Wait0 => {
                println!("Wait0");
                transition(
                    2,
                    Dig {
                        task: Move::new_task(),
                    },
                )
            }
            Dig { ref mut task } => {
                println!("Dig");
                run_task(task, Done, Init)
            }
            Done => success(),
        };
        state_update(&mut self.state, pair)
    }
}

struct TaskSequence {
    tasks: VecDeque<Box<dyn Task>>,
}

impl TaskSequence {
    fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }
}

impl Task for TaskSequence {
    fn update(&mut self) -> Status {
        if let Some(task) = self.tasks.front_mut() {
            match task.update() {
                Status::Active(n) => Status::Active(n),
                Status::Failure => Status::Failure,
                Status::Success => {
                    self.tasks.pop_front();
                    Status::Active(0)
                }
            }
        } else {
            Status::Success
        }
    }
}

fn main() {
    let mut discard_list = Vec::new();
    let mut active_tasks = Vec::<(u64, Box<dyn Task>)>::new();
    active_tasks.push((0 as u64, Box::new(Move::new())));

    let mut seq = Box::new(TaskSequence::new());
    seq.tasks.push_back(Move::new_task());
    seq.tasks.push_back(Dig::new_task());
    active_tasks.push((0, seq));

    let mut frame: u64 = 0;
    while frame < 32 {
        println!("---- frame {} ----", frame);

        for index in 0..active_tasks.len() {
            let (ref mut delay, task) = &mut active_tasks[index];
            if *delay > 0 {
                println!("(task pausing for {}...)", delay);
                *delay -= 1;
                continue;
            }

            match task.update() {
                Status::Active(wait) => {
                    println!("Should wait {}", wait);
                    *delay = wait;
                }
                Status::Failure => {
                    println!("Task failed.");
                    discard_list.push(index);
                }
                Status::Success => {
                    println!("Task succeeded.");
                    discard_list.push(index);
                }
            }
        }

        // Note: this is correct only because the discard list is guarenteed to be in
        // ascending order, so the iteration is in descending order, and swaps can
        // thus only occur with tasks that are not being discarded (i.e. the dicard_list
        // indices can't be corrupted by the swaps).
        while let Some(i) = discard_list.pop() {
            let last = active_tasks.len() - 1;
            active_tasks.swap(i, last);
            active_tasks.pop();
        }

        frame += 1;
    }
}
