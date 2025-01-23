use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use crate::component::{Component, HandleResult};
use crate::render::Message;
use oombak_sim::sim::{self, SimulationResult};

use crossterm::event::{Event, KeyCode, KeyEvent};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear};
use ratatui::Frame;

use super::models::SimulationSpec;
use super::{CommandLine, InstanceHierViewer, SignalsViewer, WaveViewer};

pub struct Root {
    message_tx: Sender<Message>,
    request_tx: Sender<sim::Request>,
    signals_viewer: SignalsViewer,
    wave_viewer: WaveViewer,
    instance_hier_viewer: Arc<RwLock<InstanceHierViewer>>,
    command_line: Arc<RwLock<CommandLine>>,
    focused_child: Option<Child>,
    simulation_spec: SimulationSpec,
    reload_simulation: bool,
}

enum Child {
    CommandLine,
    InstanceHierView,
}

impl Root {
    pub fn new(
        message_tx: Sender<Message>,
        request_tx: Sender<sim::Request>,
        command_line: Arc<RwLock<CommandLine>>,
    ) -> Self {
        let simulation_spec = SimulationSpec::default();
        Self {
            message_tx: message_tx.clone(),
            request_tx: request_tx.clone(),
            wave_viewer: WaveViewer::default().simulation(simulation_spec.clone()),
            signals_viewer: SignalsViewer::default().simulation(simulation_spec.clone()),
            instance_hier_viewer: Arc::new(RwLock::new(InstanceHierViewer::new(
                message_tx.clone(),
                request_tx.clone(),
            ))),
            command_line,
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
    fn render_mut(&mut self, f: &mut Frame, rect: Rect) {
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
        if matches!(self.focused_child, Some(Child::InstanceHierView)) {
            self.render_instance_hier_viewer(f, rect);
        }
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Char('q') => {
                self.notify_quit();
                return HandleResult::Handled;
            }
            KeyCode::Right => {
                self.wave_viewer.scroll_right();
                let highlight_idx = self.wave_viewer.get_highlighted_unit_time();
                self.signals_viewer.set_highlight(highlight_idx);
            }
            KeyCode::Left => {
                self.wave_viewer.scroll_left();
                let highlight_idx = self.wave_viewer.get_highlighted_unit_time();
                self.signals_viewer.set_highlight(highlight_idx);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.signals_viewer.scroll_up();
                self.wave_viewer.scroll_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.signals_viewer.scroll_down();
                self.wave_viewer.scroll_down();
            }
            KeyCode::Char(':') => {
                self.focused_child = Some(Child::CommandLine);
                self.try_propagate_event(&Event::Key(*key_event));
            }
            KeyCode::Char('s') => {
                self.focused_child = Some(Child::InstanceHierView);
            }
            _ => return HandleResult::NotHandled,
        }
        self.notify_render();
        HandleResult::Handled
    }

    fn handle_resize_event(&mut self, _columns: u16, _rows: u16) -> HandleResult {
        self.notify_render();
        HandleResult::Handled
    }

    fn set_focus_to_self(&mut self) {
        if matches!(self.focused_child, Some(Child::InstanceHierView)) {
            self.notify_render();
        }
        self.focused_child = None;
    }

    fn try_propagate_event(&mut self, event: &Event) -> HandleResult {
        if let Some(child) = &self.focused_child {
            match child {
                Child::CommandLine => self.command_line.write().unwrap().handle_event(event),
                Child::InstanceHierView => self
                    .instance_hier_viewer
                    .write()
                    .unwrap()
                    .handle_event(event),
            }
        } else {
            HandleResult::NotHandled
        }
    }

    fn render(&self, _f: &mut Frame, _rect: Rect) {}
}

impl Root {
    fn render_signals_viewer(&mut self, f: &mut Frame, rect: Rect) {
        self.signals_viewer.render_mut(f, rect);
    }

    fn render_wave_viewer(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::new().borders(Borders::LEFT);
        let inner = block.inner(rect);
        f.render_widget(block, rect);
        self.wave_viewer.render_mut(f, inner);
    }

    fn render_instance_hier_viewer(&self, f: &mut Frame, rect: Rect) {
        let popup_area = Self::get_popup_area(rect);
        let block = Block::new().borders(Borders::ALL);
        f.render_widget(Clear, popup_area);
        self.instance_hier_viewer
            .write()
            .unwrap()
            .render_mut_with_block(f, popup_area, block);
    }

    fn render_command_line(&self, f: &mut Frame, rect: Rect) {
        self.command_line.read().unwrap().render(f, rect);
    }

    fn get_popup_area(rect: Rect) -> Rect {
        let chunks = Layout::vertical(vec![
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(rect);
        let chunks = Layout::horizontal(vec![
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(5),
        ])
        .split(chunks[1]);
        chunks[1]
    }
}

impl sim::Listener for Root {
    fn on_receive_reponse(&mut self, response: &sim::Response) {
        match response {
            sim::Response::RunResult(Ok(_)) => self.request_simulation_result(),
            sim::Response::LoadResult(Ok(loaded_dut))
            | sim::Response::ModifyProbedPointsResult(Ok(loaded_dut)) => {
                self.instance_hier_viewer
                    .write()
                    .unwrap()
                    .set_loaded_dut(loaded_dut);
                self.reload_simulation = true;
                self.request_simulation_result();
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
