use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};

use crate::{component::Component, widgets::Waveform};

use super::models::SimulationSpec;

#[derive(Default)]
pub struct WaveViewer {
    simulation: SimulationSpec,
    highlight_idx: u16,
}

impl WaveViewer {
    pub fn simulation(mut self, simulation: SimulationSpec) -> Self {
        self.simulation = simulation;
        self
    }

    pub fn set_highlight(&mut self, idx: u16) {
        self.highlight_idx = idx;
    }

    fn get_layout_constraints(&self) -> Vec<Constraint> {
        self.simulation
            .wave_specs
            .iter()
            .map(|spec| Constraint::Length(spec.height * 2 + 2))
            .collect()
    }
}

impl Component for WaveViewer {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let block = Block::new().borders(Borders::TOP);
        let inner = block.inner(rect);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(self.get_layout_constraints())
            .split(inner);
        for (i, wave_spec) in self.simulation.wave_specs.iter().enumerate() {
            let block = Block::new().borders(Borders::BOTTOM);
            let waveform = Waveform::from(wave_spec)
                .width(self.simulation.zoom)
                .highlight(self.highlight_idx);
            f.render_widget(waveform, block.inner(layout[i]));
            f.render_widget(block, layout[i]);
        }
        f.render_widget(block, rect);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {
        todo!()
    }
}
