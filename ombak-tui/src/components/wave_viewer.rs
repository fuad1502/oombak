use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};

use crate::{backend::Wave, component::Component, utils::bitvec_str, widgets::Waveform};

pub struct WaveViewer {
    waves: Vec<Wave>,
    height: u16,
    zoom: u8,
}

impl Default for WaveViewer {
    fn default() -> Self {
        Self {
            waves: vec![],
            height: 1,
            zoom: 1,
        }
    }
}

impl WaveViewer {
    pub fn waves(mut self, waves: Vec<Wave>) -> Self {
        self.waves = waves;
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn zoom(mut self, zoom: u8) -> Self {
        self.zoom = zoom;
        self
    }
}

impl Component for WaveViewer {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let block = Block::new().borders(Borders::TOP);
        let inner = block.inner(rect);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2 * self.height + 2);
                self.waves.len()
            ])
            .split(inner);
        for (i, wave) in self.waves.iter().enumerate() {
            let option = bitvec_str::Option {
                width: wave.width,
                ..Default::default()
            };
            let block = Block::new().borders(Borders::BOTTOM);
            let waveform = Waveform::new(&wave.values, self.height, self.zoom, option);
            f.render_widget(waveform, block.inner(layout[i]));
            f.render_widget(block, layout[i]);
        }
        f.render_widget(block, rect);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {
        todo!()
    }
}
