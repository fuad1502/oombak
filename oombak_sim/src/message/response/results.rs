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
    pub compact_values: Vec<CompactWaveValue>,
    start_times: Vec<usize>,
}

#[derive(Clone)]
pub struct CompactWaveValue {
    value: BitVec<u32>,
    duration: usize,
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
            compact_values: vec![],
            start_times: vec![],
        }
    }
}

impl Wave {
    pub fn slice(&self, start_time: usize, end_time: usize) -> Option<Vec<CompactWaveValue>> {
        let (start_idx, start_offset) = match self.find(start_time) {
            Some((a, b)) => (a, b),
            None => return None,
        };
        let (end_idx, end_offset) = match self.find(end_time) {
            Some((a, b)) => (a, b),
            None => return None,
        };

        let mut slice = vec![];
        let first_value = self.compact_values[start_idx].value.clone();
        let first_duration = self.compact_values[start_idx].duration - start_offset;
        let last_value = self.compact_values[end_idx].value.clone();
        let last_duration = end_offset + 1;

        if start_idx != end_idx {
            slice.push(CompactWaveValue::new(first_value, first_duration));
        } else {
            let duration = end_offset - start_offset + 1;
            slice.push(CompactWaveValue::new(first_value, duration));
        }

        if end_idx - start_idx > 1 {
            let mut middle_part = Vec::from(&self.compact_values[start_idx + 1..end_idx]);
            slice.append(&mut middle_part);
        }

        if end_idx != start_idx {
            slice.push(CompactWaveValue::new(last_value, last_duration));
        }

        Some(slice)
    }

    pub fn append(&mut self, compact_value: CompactWaveValue) {
        if compact_value.duration == 0 {
            return;
        }

        if let (Some(last_compact_value), Some(last_start_time)) =
            (self.compact_values.last_mut(), self.start_times.last())
        {
            if last_compact_value.value == compact_value.value {
                last_compact_value.duration += compact_value.duration;
            } else {
                self.start_times
                    .push(last_start_time + last_compact_value.duration);
                self.compact_values.push(compact_value);
            }
        } else {
            self.compact_values.push(compact_value);
            self.start_times.push(0);
        }
    }

    pub fn end_time(&self) -> usize {
        if let (Some(last_start_time), Some(last_compact_value)) =
            (self.start_times.last(), self.compact_values.last())
        {
            last_start_time + last_compact_value.duration - 1
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.compact_values.is_empty()
    }

    pub fn at(&self, time: usize) -> Option<&BitVec<u32>> {
        if let Some((idx, _)) = self.find(time) {
            Some(&self.compact_values[idx].value)
        } else {
            None
        }
    }

    fn find(&self, time: usize) -> Option<(usize, usize)> {
        if self.start_times.is_empty() || time > self.end_time() {
            return None;
        }
        let index = self.start_times.partition_point(|t| *t <= time) - 1;
        let offset = time - self.start_times[index];
        Some((index, offset))
    }
}

impl CompactWaveValue {
    pub fn new(value: BitVec<u32>, duration: usize) -> Self {
        Self { value, duration }
    }

    pub fn value(&self) -> &BitVec<u32> {
        &self.value
    }

    pub fn duration(&self) -> usize {
        self.duration
    }
}
