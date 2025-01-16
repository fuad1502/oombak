use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use crate::component::Component;
use crate::render::Message;
use oombak_sim::sim::{self, SimulationResult};

use crossterm::event::{Event, KeyCode, KeyEvent};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use super::models::SimulationSpec;
use super::{CommandLine, SignalsViewer, WaveViewer};

pub struct Root {
    message_tx: Sender<Message>,
    request_tx: Sender<sim::Request>,
    signals_viewer: SignalsViewer,
    wave_viewer: WaveViewer,
    command_line: Arc<RwLock<CommandLine>>,
    highlight_idx: u16,
    focused_child: Option<Child>,
    simulation_spec: SimulationSpec,
    reload_simulation: bool,
}

enum Child {
    CommandLine,
}

impl Root {
    pub fn new(
        message_tx: Sender<Message>,
        request_tx: Sender<sim::Request>,
        command_line: Arc<RwLock<CommandLine>>,
    ) -> Self {
        let simulation_spec = SimulationSpec::default();
        Self {
            message_tx,
            request_tx,
            wave_viewer: WaveViewer::default().simulation(simulation_spec.clone()),
            signals_viewer: SignalsViewer::default().simulation(simulation_spec.clone()),
            command_line,
            highlight_idx: 0,
            focused_child: None,
            simulation_spec,
            reload_simulation: false,
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

impl sim::Listener for Root {
    fn on_receive_reponse(&mut self, response: &sim::Response) {
        match response {
            sim::Response::RunResult(Ok(_)) => self.request_simulation_result(),
            sim::Response::LoadResult(Ok(_)) => {
                self.request_simulation_result();
                self.reload_simulation = true;
            }
            sim::Response::SimulationResult(Ok(simulation_result)) => {
                self.update_simulation_spec(simulation_result);
                self.notify_render();
            }
            _ => (),
        }
    }
}

impl Root {
    fn update_simulation_spec(&mut self, simulation_result: &SimulationResult) {
        if self.reload_simulation {
            self.simulation_spec = SimulationSpec::new(simulation_result);
        } else {
            self.simulation_spec.update(simulation_result);
        }
        self.signals_viewer
            .set_simulation(self.simulation_spec.clone());
        self.wave_viewer
            .set_simulation(self.simulation_spec.clone());
    }

    fn request_simulation_result(&self) {
        self.request_tx
            .send(sim::Request::GetSimulationResult)
            .unwrap();
    }
}
