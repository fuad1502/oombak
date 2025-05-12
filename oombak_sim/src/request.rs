use std::path::PathBuf;

use bitvec::vec::BitVec;

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
