use crate::utils::bitvec_str;

#[derive(Default, Clone)]
pub struct SimulationSpec {
    pub wave_specs: Vec<WaveSpec>,
    pub total_time: usize,
    pub time_step_ps: usize,
    pub zoom: u8,
}

#[derive(Clone)]
pub struct WaveSpec {
    pub wave: oombak_sim::response::Wave,
    pub height: u16,
    pub format: bitvec_str::Format,
    pub signed: bool,
}

impl SimulationSpec {
    pub fn new(simulation_result: &oombak_sim::response::SimulationResult) -> Self {
        let mut spec = SimulationSpec {
            wave_specs: vec![],
            total_time: simulation_result.current_time,
            time_step_ps: 1,
            zoom: 1,
        };
        spec.wave_specs = simulation_result
            .waves
            .iter()
            .map(|w| WaveSpec {
                wave: w.clone(),
                height: 1,
                format: bitvec_str::Format::Binary,
                signed: true,
            })
            .collect();
        spec
    }

    pub fn update(&mut self, simulation_result: &oombak_sim::response::SimulationResult) {
        self.time_step_ps = simulation_result.time_step_ps;
        self.total_time = simulation_result.current_time;
        self.wave_specs = simulation_result
            .waves
            .iter()
            .zip(self.wave_specs.iter_mut())
            .map(|(w, s)| WaveSpec {
                wave: w.clone(),
                ..s.clone()
            })
            .collect();
    }

    pub fn is_empty(&self) -> bool {
        self.wave_specs.is_empty()
    }
}
