use ratatui::layout::{Constraint, Direction, Layout};

use crate::component::Component;

use super::Button;

pub struct Toolbar {
    load_button: Button,
    run_button: Button,
    restart_button: Button,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self {
            load_button: Button::default().icon("\u{f07c}"),
            run_button: Button::default().icon("\u{ead3}"),
            restart_button: Button::default().icon("\u{ead2}"),
        }
    }
}

impl Component for Toolbar {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(rect);
        self.load_button.render(f, layout[0]);
        self.run_button.render(f, layout[1]);
        self.restart_button.render(f, layout[2]);
    }

    fn handle_key_event(&mut self, _key_event: &crossterm::event::KeyEvent) {
        todo!()
    }
}
