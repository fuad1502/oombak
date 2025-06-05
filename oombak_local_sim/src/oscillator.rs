use std::collections::BinaryHeap;

use bitvec::vec::BitVec;

#[derive(Default)]
pub struct OscillatorGroup {
    priority_queue: BinaryHeap<Oscillator>,
}

#[derive(Eq, PartialEq)]
pub struct Oscillator {
    signal_name: String,
    period: usize,
    next_state: State,
    next_trigger_time: usize,
    low_state_value: BitVec<u32>,
    high_state_value: BitVec<u32>,
}

#[derive(Eq, PartialEq)]
enum State {
    Low,
    High,
}

impl OscillatorGroup {
    pub fn insert(&mut self, oscillator: Oscillator) {
        self.remove(&oscillator.signal_name);
        self.priority_queue.push(oscillator);
    }

    pub fn remove(&mut self, signal_name: &str) {
        self.priority_queue.retain(|e| e.signal_name != signal_name);
    }

    pub fn next_trigger_time(&self) -> Option<usize> {
        self.priority_queue.peek().map(|e| e.next_trigger_time)
    }

    pub fn try_pop(&mut self, current_time: usize) -> Option<(String, BitVec<u32>)> {
        if let Some(oscillator) = self.priority_queue.peek() {
            if oscillator.next_trigger_time == current_time {
                let mut oscillator = self.priority_queue.pop().unwrap();
                let popped_name = oscillator.signal_name.clone();
                let popped_value = oscillator.trip();
                self.priority_queue.push(oscillator);
                return Some((popped_name, popped_value));
            }
        }
        None
    }
}

impl Oscillator {
    pub fn new(
        signal_name: String,
        period: usize,
        current_time: usize,
        low_state_value: BitVec<u32>,
        high_state_value: BitVec<u32>,
    ) -> Self {
        Self {
            signal_name,
            period,
            next_state: State::High,
            next_trigger_time: current_time + period,
            low_state_value,
            high_state_value,
        }
    }

    fn trip(&mut self) -> BitVec<u32> {
        let popped_value = match self.next_state {
            State::Low => self.low_state_value.clone(),
            State::High => self.high_state_value.clone(),
        };
        self.next_trigger_time += self.period;
        self.next_state = match self.next_state {
            State::Low => State::High,
            State::High => State::Low,
        };
        popped_value
    }
}

impl Ord for Oscillator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .next_trigger_time
            .cmp(&self.next_trigger_time)
            .then_with(|| self.signal_name.cmp(&other.signal_name))
    }
}

impl PartialOrd for Oscillator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
