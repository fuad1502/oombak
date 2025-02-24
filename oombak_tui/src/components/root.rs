use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use crate::component::{Component, HandleResult};
use crate::threads::RendererMessage;
use oombak_sim::sim::{self, SimulationResult};

use crossterm::event::{KeyCode, KeyEvent};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear};
use ratatui::Frame;

use super::models::SimulationSpec;
use super::{CommandInterpreter, FileExplorer, InstanceHierViewer, SignalsViewer, WaveViewer};

pub struct Root {
    message_tx: Sender<RendererMessage>,
    request_tx: Sender<sim::Request>,
    signals_viewer: SignalsViewer,
    wave_viewer: WaveViewer,
    instance_hier_viewer: Arc<RwLock<InstanceHierViewer>>,
    command_interpreter: Arc<RwLock<CommandInterpreter>>,
    file_explorer: Arc<RwLock<FileExplorer>>,
    focused_child: Option<Child>,
    simulation_spec: SimulationSpec,
    reload_simulation: bool,
}

enum Child {
    CommandInterpreter,
    InstanceHierView,
    FileExplorer,
}

impl Root {
    pub fn new(
        message_tx: Sender<RendererMessage>,
        request_tx: Sender<sim::Request>,
        command_interpreter: Arc<RwLock<CommandInterpreter>>,
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
            command_interpreter,
            file_explorer: Arc::new(RwLock::new(FileExplorer::new(
                message_tx.clone(),
                request_tx.clone(),
            ))),
            focused_child: None,
            simulation_spec,
            reload_simulation: false,
        }
    }

    fn notify_render(&self) {
        self.message_tx.send(RendererMessage::Render).unwrap();
    }

    fn notify_quit(&self) {
        self.message_tx.send(RendererMessage::Quit).unwrap();
    }
}

impl Component for Root {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
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
        self.render_command_interpreter(f, rect, main_layout_v[1]);
        if matches!(self.focused_child, Some(Child::InstanceHierView)) {
            self.render_instance_hier_viewer(f, rect);
        }
        if matches!(self.focused_child, Some(Child::FileExplorer)) {
            self.render_file_explorer(f, rect);
        }
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Char('q') => {
                self.notify_quit();
                return HandleResult::Handled;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.wave_viewer.scroll_right();
                self.update_signal_viewer_highlight();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.wave_viewer.scroll_left();
                self.update_signal_viewer_highlight();
            }
            KeyCode::Char('+') | KeyCode::Char('z') => {
                self.simulation_spec.zoom = self.simulation_spec.zoom.saturating_add(1);
                self.wave_viewer.zoom_in();
                self.update_signal_viewer_highlight();
            }
            KeyCode::Char('-') | KeyCode::Char('x') => {
                self.simulation_spec.zoom = self.simulation_spec.zoom.saturating_sub(1);
                self.wave_viewer.zoom_out();
                self.update_signal_viewer_highlight();
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
                self.focused_child = Some(Child::CommandInterpreter);
                self.command_interpreter.write().unwrap().set_line_mode();
            }
            KeyCode::Char('t') => {
                self.focused_child = Some(Child::CommandInterpreter);
                self.command_interpreter.write().unwrap().set_window_mode();
            }
            KeyCode::Char('s') => {
                self.focused_child = Some(Child::InstanceHierView);
            }
            KeyCode::Char('o') => {
                self.focused_child = Some(Child::FileExplorer);
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

    fn handle_focus_gained(&mut self) {
        self.notify_render();
        self.focused_child = None;
    }

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>> {
        match self.focused_child {
            Some(Child::CommandInterpreter) => Some(self.command_interpreter.clone()),
            Some(Child::InstanceHierView) => Some(self.instance_hier_viewer.clone()),
            Some(Child::FileExplorer) => Some(self.file_explorer.clone()),
            None => None,
        }
    }
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
            .render_with_block(f, popup_area, block);
    }

    fn render_file_explorer(&self, f: &mut Frame, rect: Rect) {
        let popup_area = Self::get_popup_area(rect);
        let block = Block::new().borders(Borders::ALL);
        f.render_widget(Clear, popup_area);
        self.file_explorer
            .write()
            .unwrap()
            .render_with_block(f, popup_area, block);
    }

    fn render_command_interpreter(&self, f: &mut Frame, window_area: Rect, line_area: Rect) {
        self.render_interpreter_on_line(f, line_area);
        if matches!(self.focused_child, Some(Child::CommandInterpreter))
            && self.command_interpreter.read().unwrap().is_window_mode()
        {
            self.render_interpreter_on_window(f, window_area);
        }
    }

    fn render_interpreter_on_line(&self, f: &mut Frame, rect: Rect) {
        self.command_interpreter
            .write()
            .unwrap()
            .render_on_line(f, rect);
    }

    fn render_interpreter_on_window(&self, f: &mut Frame, rect: Rect) {
        let popup_area = Self::get_popup_area(rect);
        let block = Block::new().borders(Borders::ALL);
        let inner = block.inner(popup_area);
        f.render_widget(Clear, popup_area);
        f.render_widget(block, popup_area);
        self.command_interpreter
            .write()
            .unwrap()
            .render_on_window(f, inner);
    }

    fn update_signal_viewer_highlight(&mut self) {
        let highlight_idx = self.wave_viewer.get_highlighted_unit_time();
        self.signals_viewer.set_highlight(highlight_idx);
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
            self.reload_simulation = false;
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
