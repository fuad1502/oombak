pub mod results;

pub use results::{LoadedDut, SimulationResult, Wave};

pub struct Response<'a> {
    pub id: usize,
    pub payload: Payload<'a>,
}

pub enum Payload<'a> {
    Result(Results<'a>),
    Error(Box<dyn std::error::Error + Send + Sync>),
    Notification(Notifications),
}

pub enum Results<'a> {
    CurrentTime(u64),
    LoadedDut(LoadedDut),
    SimulationResult(&'a SimulationResult),
    Empty,
}

pub enum Errors {
    Generic(String),
}

pub enum Notifications {
    Progress(Percentage, String),
    Generic(String),
}

pub struct Percentage {
    num_of_steps: usize,
    completed_steps: usize,
}

impl<'a> From<&'a SimulationResult> for Payload<'a> {
    fn from(value: &'a SimulationResult) -> Self {
        Payload::Result(Results::SimulationResult(value))
    }
}

impl From<LoadedDut> for Payload<'_> {
    fn from(value: LoadedDut) -> Self {
        Payload::Result(Results::LoadedDut(value))
    }
}

impl Payload<'_> {
    pub fn current_time(current_time: u64) -> Self {
        Payload::Result(Results::CurrentTime(current_time))
    }

    pub fn empty() -> Self {
        Payload::Result(Results::Empty)
    }
}

impl Percentage {
    pub fn new(num_of_steps: usize) -> Self {
        Self {
            num_of_steps,
            completed_steps: 0,
        }
    }

    pub fn increment(&mut self) {
        self.completed_steps += 1;
    }

    pub fn value(&self) -> f32 {
        self.completed_steps as f32 / self.num_of_steps as f32
    }
}
