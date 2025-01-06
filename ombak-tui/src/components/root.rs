use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use crate::backend::simulator::Wave;
use crate::component::Component;
use crate::render::Message;
use crate::utils::bitvec_str;

use bitvec::vec::BitVec;
use crossterm::event::{Event, KeyCode, KeyEvent};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use super::models::{SimulationSpec, WaveSpec};
use super::{CommandLine, SignalsViewer, WaveViewer};

pub struct Root {
    message_tx: Sender<Message>,
    signals_viewer: SignalsViewer,
    wave_viewer: WaveViewer,
    command_line: Arc<RwLock<CommandLine>>,
    highlight_idx: u16,
    focused_child: Option<Child>,
}

enum Child {
    CommandLine,
}

impl Root {
    pub fn new(message_tx: Sender<Message>, command_line: Arc<RwLock<CommandLine>>) -> Self {
        Self {
            message_tx: message_tx.clone(),
            wave_viewer: WaveViewer::default().simulation(Self::get_waves()),
            signals_viewer: SignalsViewer::default().simulation(Self::get_waves()),
            command_line,
            highlight_idx: 0,
            focused_child: None,
        }
    }

    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
    }

    fn notify_quit(&self) {
        self.message_tx.send(Message::Quit).unwrap();
    }

    fn get_waves() -> SimulationSpec {
        let waves = vec![
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
        ];
        let wave_specs = waves
            .into_iter()
            .map(|wave| WaveSpec {
                wave,
                height: 1,
                format: bitvec_str::Format::Binary,
                signed: true,
            })
            .collect();
        SimulationSpec {
            wave_specs,
            time_step_ps: 10,
            zoom: 10,
        }
    }
}

impl Component for Root {
    fn render(&self, f: &mut Frame, rect: Rect) {
        let main_layout_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(0), Constraint::Length(1)])
            .split(rect);
        let sub_layout_h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(main_layout_v[0]);
        self.render_signals_viewer(f, sub_layout_h[0]);
        self.render_wave_viewer(f, sub_layout_h[1]);
        self.render_command_line(f, main_layout_v[1]);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Char('q') => {
                self.notify_quit();
                return true;
            }
            KeyCode::Right => {
                self.highlight_idx = u16::saturating_add(self.highlight_idx, 1);
                self.set_highlight();
            }
            KeyCode::Left => {
                self.highlight_idx = u16::saturating_sub(self.highlight_idx, 1);
                self.set_highlight();
            }
            KeyCode::Char(':') => {
                self.focused_child = Some(Child::CommandLine);
                self.propagate_event(&Event::Key(*key_event));
            }
            _ => return false,
        }
        self.notify_render();
        true
    }

    fn set_focus(&mut self) {
        self.focused_child = None;
    }

    fn propagate_event(&mut self, event: &Event) -> bool {
        if let Some(child) = &self.focused_child {
            match child {
                Child::CommandLine => self.command_line.write().unwrap().handle_event(event),
            }
        } else {
            false
        }
    }

    fn get_focused_child(&mut self) -> Option<&mut dyn Component> {
        None
    }
}

impl Root {
    fn set_highlight(&mut self) {
        self.signals_viewer.set_highlight(self.highlight_idx);
        self.wave_viewer.set_highlight(self.highlight_idx);
    }

    fn render_signals_viewer(&self, f: &mut Frame, rect: Rect) {
        self.signals_viewer.render(f, rect);
    }

    fn render_wave_viewer(&self, f: &mut Frame, rect: Rect) {
        let block = Block::new().borders(Borders::LEFT);
        self.wave_viewer.render_with_block(f, rect, block);
    }

    fn render_command_line(&self, f: &mut Frame, rect: Rect) {
        self.command_line.read().unwrap().render(f, rect);
    }
}
