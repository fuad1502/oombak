use crate::{backend::simulator::Wave, utils::bitvec_str};

#[derive(Default)]
pub struct SimulationSpec {
    pub wave_specs: Vec<WaveSpec>,
    pub time_step_ps: usize,
    pub zoom: u8,
}

pub struct WaveSpec {
    pub wave: Wave,
    pub height: u16,
    pub format: bitvec_str::Format,
    pub signed: bool,
}
