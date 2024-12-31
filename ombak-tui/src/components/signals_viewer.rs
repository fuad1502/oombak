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
    highlight_idx: usize,
}

impl SignalsViewer {
    pub fn simulation(mut self, simulation: SimulationSpec) -> Self {
        self.simulation = simulation;
        self
    }

    fn format(&self, wave_spec: &WaveSpec) -> String {
        let value = &wave_spec.wave.values[self.highlight_idx / (self.simulation.zoom as usize)];
        let option = bitvec_str::Option::from(wave_spec);
        let value = utils::bitvec_str::from(value, &option);
        let vertical_alignments =
            std::iter::repeat_n("\n", wave_spec.height as usize).collect::<String>();
        format!(
            "{vertical_alignments}{} [{}:0] ({value})",
            wave_spec.wave.signal_name, wave_spec.wave.width
        )
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
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let block = Block::new().borders(Borders::TOP | Borders::RIGHT);
        let inner = block.inner(rect);
        let layout = Layout::vertical(self.get_layout_constraints()).split(inner);
        for (i, wave_spec) in self.simulation.wave_specs.iter().enumerate() {
            let block = Block::new().borders(Borders::BOTTOM);
            let text = self.format(wave_spec);
            let paragraph = Paragraph::new(text).right_aligned().block(block);
            f.render_widget(paragraph, layout[i]);
        }
        f.render_widget(block, rect);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {
        todo!()
    }
}
