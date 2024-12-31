use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
};

use crate::{backend::Wave, component::Component};

pub struct SignalsViewer {
    waves: Vec<Wave>,
    height: u16,
}

impl Default for SignalsViewer {
    fn default() -> Self {
        Self {
            waves: vec![],
            height: 1,
        }
    }
}

impl SignalsViewer {
    pub fn waves(mut self, waves: Vec<Wave>) -> Self {
        self.waves = waves;
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    fn format(wave: &Wave) -> String {
        format!("{} [{}:0]", wave.signal_name, wave.width)
    }
}

impl Component for SignalsViewer {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let block = Block::new().borders(Borders::TOP | Borders::RIGHT);
        let inner = block.inner(rect);
        let layout = Layout::vertical(vec![
            Constraint::Length(2 * self.height + 2);
            self.waves.len()
        ])
        .split(inner);
        for (i, wave) in self.waves.iter().enumerate() {
            let block = Block::new().borders(Borders::BOTTOM);
            let text = Self::format(wave);
            let paragraph = Paragraph::new(text).right_aligned().block(block);
            f.render_widget(paragraph, layout[i]);
        }
        f.render_widget(block, rect);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {
        todo!()
    }
}
