use oombak_rs::{parser::InstanceNode, probe::Probe};

use bitvec::vec::BitVec;

pub struct LoadedDut {
    pub root_node: InstanceNode,
    pub probed_points: Vec<String>,
}

#[derive(Clone, Default)]
pub struct SimulationResult {
    pub waves: Vec<Wave>,
    pub time_step_ps: usize,
    pub current_time: usize,
}

#[derive(Clone)]
pub struct Wave {
    pub signal_name: String,
    pub width: usize,
    pub values: Vec<(BitVec<u32>, usize, usize)>,
}

impl From<&Probe> for LoadedDut {
    fn from(probe: &Probe) -> Self {
        let probed_points = probe
            .get_probed_points()
            .iter()
            .map(|p| p.path().to_string())
            .collect();
        let root_node = probe.root_node().clone();
        LoadedDut {
            probed_points,
            root_node,
        }
    }
}

impl From<oombak_rs::dut::Signal> for Wave {
    fn from(signal: oombak_rs::dut::Signal) -> Self {
        Wave {
            signal_name: signal.name,
            width: signal.width as usize,
            values: vec![],
        }
    }
}

impl Wave {
    pub fn value_idx_at(&self, time: usize) -> Option<(usize, usize)> {
        match self.values.binary_search_by(|v| (v.1).cmp(&time)) {
            Ok(idx) => Some((idx, 0)),
            Err(0) => None,
            Err(idx) => {
                let offset = time - self.values[idx - 1].1;
                if offset < self.values[idx - 1].2 {
                    Some((idx - 1, offset))
                } else {
                    None
                }
            }
        }
    }
}
