use std::path::PathBuf;

use bitvec::vec::BitVec;
use rand::RngCore;

use super::Message;

#[derive(Clone)]
pub struct Request {
    pub id: usize,
    pub payload: Payload,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Payload {
    Run(usize),
    SetSignal(String, BitVec<u32>),
    Load(PathBuf),
    ModifyProbedPoints(ProbePointsModification),
    GetSimulationResult,
    Terminate,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProbePointsModification {
    pub to_add: Vec<String>,
    pub to_remove: Vec<String>,
}

impl Request {
    pub fn run(duration: usize) -> Message {
        let id = Self::random_id();
        let payload = Payload::Run(duration);
        Message::Request(Self { id, payload })
    }

    pub fn set_signal(signal_name: String, value: BitVec<u32>) -> Message {
        let id = Self::random_id();
        let payload = Payload::SetSignal(signal_name, value);
        Message::Request(Self { id, payload })
    }

    pub fn load(sv_path: PathBuf) -> Message {
        let id = Self::random_id();
        let payload = Payload::Load(sv_path);
        Message::Request(Self { id, payload })
    }

    pub fn modify_probe_points(probe_points_modifications: ProbePointsModification) -> Message {
        let id = Self::random_id();
        let payload = Payload::ModifyProbedPoints(probe_points_modifications);
        Message::Request(Self { id, payload })
    }

    pub fn get_simulation_result() -> Message {
        let id = Self::random_id();
        let payload = Payload::GetSimulationResult;
        Message::Request(Self { id, payload })
    }

    pub fn terminate() -> Message {
        let id = Self::random_id();
        let payload = Payload::Terminate;
        Message::Request(Self { id, payload })
    }

    fn random_id() -> usize {
        let mut rng = rand::rng();
        rng.next_u32() as usize
    }
}
