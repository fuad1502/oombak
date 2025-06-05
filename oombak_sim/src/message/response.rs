mod results;

pub use results::{LoadedDut, SimulationResult, Wave};

pub struct Response {
    pub id: usize,
    pub payload: Payload,
}

pub enum Payload {
    Result(Results),
    Error(Box<dyn std::error::Error + Send + Sync>),
    Notification(Notifications),
}

pub enum Results {
    CurrentTime(usize),
    LoadedDut(LoadedDut),
    SimulationResult(SimulationResult),
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

impl Response {
    pub fn result(&self) -> Option<&Results> {
        if let Payload::Result(result) = &self.payload {
            Some(result)
        } else {
            None
        }
    }
}

impl From<SimulationResult> for Payload {
    fn from(value: SimulationResult) -> Self {
        Payload::Result(Results::SimulationResult(value))
    }
}

impl From<LoadedDut> for Payload {
    fn from(value: LoadedDut) -> Self {
        Payload::Result(Results::LoadedDut(value))
    }
}

impl Payload {
    pub fn generic_notification(message: String) -> Self {
        Payload::Notification(Notifications::Generic(message))
    }

    pub fn current_time(current_time: usize) -> Self {
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

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.completed_steps, self.num_of_steps)
    }
}

impl std::fmt::Display for Notifications {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Notifications::Progress(percentage, message) => write!(f, "{message} ({percentage})"),
            Notifications::Generic(message) => write!(f, "{message}"),
        }
    }
}
