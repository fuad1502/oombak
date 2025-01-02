use std::sync::mpsc::Sender;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{style::Stylize, widgets::Paragraph};

use crate::{component::Component, render::Message};

pub struct CommandLine {
    message_tx: Sender<Message>,
    text: String,
    result_history: Vec<Result<String, String>>,
    state: State,
}

#[derive(PartialEq)]
enum State {
    Active,
    NotActive,
}

impl CommandLine {
    pub fn new(message_tx: Sender<Message>) -> Self {
        Self {
            message_tx,
            text: "".to_string(),
            result_history: vec![],
            state: State::NotActive,
        }
    }
}

impl Component for CommandLine {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let paragraph = match self.state {
            State::Active => Paragraph::new(self.text.clone()).black().on_light_yellow(),
            State::NotActive => match self.result_history.last() {
                Some(Ok(res)) => Paragraph::new(res.clone()).green().on_black(),
                Some(Err(res)) => Paragraph::new(res.clone()).red().on_black(),
                _ => Paragraph::new("").on_black(),
            },
        };
        f.render_widget(paragraph, rect);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Esc => {
                self.state = State::NotActive;
                self.notify_render();
                false
            }
            KeyCode::Enter => {
                self.state = State::NotActive;
                self.execute_command();
                self.notify_render();
                false
            }
            KeyCode::Char(':') => {
                self.state = State::Active;
                self.text = ":".to_string();
                self.notify_render();
                true
            }
            KeyCode::Char(c) if self.state == State::Active => {
                self.text += &format!("{c}");
                self.notify_render();
                true
            }
            KeyCode::Backspace if self.state == State::Active && self.text.len() > 1 => {
                self.text.pop();
                self.notify_render();
                true
            }
            _ => true,
        }
    }

    fn set_focus(&mut self) {}

    fn get_focused_child(&mut self) -> Option<&mut dyn Component> {
        None
    }
}

impl CommandLine {
    fn execute_command(&mut self) {
        self.result_history
            .push(Ok(format!("executed: {}", &self.text[1..])));
    }

    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
    }
}
