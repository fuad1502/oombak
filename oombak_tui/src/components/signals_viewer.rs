use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    component::{Component, HandleResult},
    utils::{self, bitvec_str},
};

use super::models::{SimulationSpec, WaveSpec};

#[derive(Default)]
pub struct SignalsViewer {
    simulation: SimulationSpec,
    highlight_idx: u16,
}

impl SignalsViewer {
    pub fn simulation(mut self, simulation: SimulationSpec) -> Self {
        self.simulation = simulation;
        self
    }

    pub fn set_highlight(&mut self, idx: u16) {
        self.highlight_idx = idx;
    }

    pub fn set_simulation(&mut self, simulation: SimulationSpec) {
        self.simulation = simulation;
    }

    fn format(&self, wave_spec: &WaveSpec) -> String {
        let vertical_alignments =
            std::iter::repeat_n("\n", wave_spec.height as usize).collect::<String>();
        format!(
            "{vertical_alignments}{} [{}:0] ({})",
            wave_spec.wave.signal_name,
            wave_spec.wave.width,
            self.get_highlighted_value(wave_spec)
        )
    }

    fn get_highlighted_value(&self, wave_spec: &WaveSpec) -> String {
        let waveform_length = wave_spec.height * 2 + 1 + self.simulation.zoom as u16;
        if let Some(value) = wave_spec
            .wave
            .values
            .get(self.highlight_idx as usize / waveform_length as usize)
        {
            let option = bitvec_str::Option::from(wave_spec);
            utils::bitvec_str::from(value, &option)
        } else {
            "x".to_string()
        }
    }

    fn get_layout_constraints(&self) -> Vec<Constraint> {
        self.simulation
            .wave_specs
            .iter()
            .map(|spec| Constraint::Length(spec.height * 2 + 2))
            .collect()
    }
}

impl Component for SignalsViewer {
    fn render(&self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let layout = Layout::vertical(self.get_layout_constraints()).split(rect);
        for (i, wave_spec) in self.simulation.wave_specs.iter().enumerate() {
            let block = Block::new().borders(Borders::BOTTOM);
            let text = self.format(wave_spec);
            let paragraph = Paragraph::new(text).right_aligned().block(block);
            f.render_widget(paragraph, layout[i]);
        }
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) -> HandleResult {
        HandleResult::NotHandled
    }

    fn try_propagate_event(&mut self, _event: &crossterm::event::Event) -> HandleResult {
        HandleResult::NotHandled
    }

    fn set_focus_to_self(&mut self) {}
}
