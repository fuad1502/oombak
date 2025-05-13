use std::path::PathBuf;

use bitvec::vec::BitVec;
use rand::RngCore;

pub struct Request {
    pub id: usize,
    pub payload: Payload,
}

pub enum Payload {
    Run(u64),
    SetSignal(String, BitVec<u32>),
    Load(PathBuf),
    ModifyProbedPoints(ProbePointsModification),
    GetSimulationResult,
    Terminate,
}

pub struct ProbePointsModification {
    pub to_add: Vec<String>,
    pub to_remove: Vec<String>,
}

impl Request {
    pub fn run(duration: u64) -> Self {
        let id = Self::random_id();
        let payload = Payload::Run(duration);
        Self { id, payload }
    }

    pub fn set_signal(signal_name: String, value: BitVec<u32>) -> Self {
        let id = Self::random_id();
        let payload = Payload::SetSignal(signal_name, value);
        Self { id, payload }
    }

    pub fn load(sv_path: PathBuf) -> Self {
        let id = Self::random_id();
        let payload = Payload::Load(sv_path);
        Self { id, payload }
    }

    pub fn modify_probe_points(probe_points_modifications: ProbePointsModification) -> Self {
        let id = Self::random_id();
        let payload = Payload::ModifyProbedPoints(probe_points_modifications);
        Self { id, payload }
    }

    pub fn get_simulation_result() -> Self {
        let id = Self::random_id();
        let payload = Payload::GetSimulationResult;
        Self { id, payload }
    }

    pub fn terminate() -> Self {
        let id = Self::random_id();
        let payload = Payload::Terminate;
        Self { id, payload }
    }

    fn random_id() -> usize {
        let mut rng = rand::rng();
        rng.next_u32() as usize
    }
}
