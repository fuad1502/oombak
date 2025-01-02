use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::Paragraph;

use crate::component::Component;

#[derive(Default)]
pub struct CommandLine {}

impl Component for CommandLine {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let paragraph = Paragraph::new(":cmd line");
        f.render_widget(paragraph, rect);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> bool {
        key_event.code != KeyCode::Esc
    }

    fn set_focus(&mut self) {}

    fn get_focused_child(&mut self) -> Option<&mut dyn Component> {
        None
    }
}
