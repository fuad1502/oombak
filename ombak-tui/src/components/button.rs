use ratatui::widgets::{Block, Paragraph};

use crate::component::Component;

pub struct Button {
    icon: String,
}

impl Button {
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }
}

impl Default for Button {
    fn default() -> Self {
        Self {
            icon: "x".to_string(),
        }
    }
}

impl Component for Button {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let icon = Paragraph::new(&self.icon[..])
            .centered()
            .block(Block::bordered());
        f.render_widget(icon, rect);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {
        todo!()
    }
}
