use std::sync::mpsc::Sender;

use crate::backend::Wave;
use crate::component::Component;
use crate::render::Message;

use bitvec::vec::BitVec;
use crossterm::event::{KeyCode, KeyEvent};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use super::{SignalsViewer, Toolbar, WaveViewer};

pub struct Root {
    message_tx: Sender<Message>,
    toolbar: Toolbar,
    signals_viewer: SignalsViewer,
    wave_viewer: WaveViewer,
}

impl Root {
    pub fn new(message_tx: Sender<Message>) -> Self {
        Self {
            message_tx,
            toolbar: Toolbar::default(),
            wave_viewer: WaveViewer::default().waves(Self::get_waves()).zoom(10),
            signals_viewer: SignalsViewer::default().waves(Self::get_waves()),
        }
    }

    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
    }

    fn notify_quit(&self) {
        self.message_tx.send(Message::Quit).unwrap();
    }

    fn get_waves() -> Vec<Wave> {
        vec![
            Wave {
                signal_name: "sig_1".to_string(),
                width: 2,
                values: vec![
                    BitVec::from_slice(&[0x0]),
                    BitVec::from_slice(&[0x1]),
                    BitVec::from_slice(&[0x2]),
                ],
            },
            Wave {
                signal_name: "sig_2".to_string(),
                width: 8,
                values: vec![
                    BitVec::from_slice(&[0xaa]),
                    BitVec::from_slice(&[0xfa]),
                    BitVec::from_slice(&[0xfa]),
                ],
            },
            Wave {
                signal_name: "sig_3".to_string(),
                width: 8,
                values: vec![
                    BitVec::from_slice(&[0xaa]),
                    BitVec::from_slice(&[0xaa]),
                    BitVec::from_slice(&[0xaa]),
                ],
            },
        ]
    }
}

impl Component for Root {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let layout_0 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
            .split(rect);
        let layout_1 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(layout_0[1]);
        self.toolbar.render(f, layout_0[0]);
        self.signals_viewer.render(f, layout_1[0]);
        self.wave_viewer.render(f, layout_1[1]);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if let KeyCode::Char('q') = key_event.code {
            self.notify_quit();
            return;
        }
        self.notify_render();
    }
}
