use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    component::Component,
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
        let value = &wave_spec.wave.values[self.highlight_idx as usize / waveform_length as usize];
        let option = bitvec_str::Option::from(wave_spec);
        utils::bitvec_str::from(value, &option)
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

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) -> bool {
        false
    }

    fn set_focus(&mut self) {}

    fn get_focused_child(&mut self) -> Option<&mut dyn Component> {
        None
    }
}
