use ratatui::widgets::Paragraph;

use crate::component::Component;

#[derive(Default)]
pub struct CommandLine {}

impl Component for CommandLine {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let paragraph = Paragraph::new(":cmd line");
        f.render_widget(paragraph, rect);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {}
}
