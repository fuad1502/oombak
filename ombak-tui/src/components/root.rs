use std::sync::mpsc::Sender;

use crate::component::Component;
use crate::render::Message;

use crossterm::event::{KeyCode, KeyEvent};

use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub struct Root {
    message_tx: Sender<Message>,
    count: i32,
}

impl Root {
    pub fn new(message_tx: Sender<Message>) -> Self {
        Self {
            message_tx,
            count: 0,
        }
    }

    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
    }

    fn notify_quit(&self) {
        self.message_tx.send(Message::Quit).unwrap();
    }
}

impl Component for Root {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(
            Paragraph::new(format!("Hello, world! {}", self.count)),
            rect,
        );
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.count += 1,
            KeyCode::Down => self.count -= 1,
            KeyCode::Char('q') => {
                self.notify_quit();
                return;
            }
            _ => (),
        }
        self.notify_render();
    }
}
