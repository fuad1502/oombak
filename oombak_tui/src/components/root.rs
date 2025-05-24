use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use crate::component::{Component, HandleResult};
use crate::threads::{simulator_request_dispatcher, RendererMessage};
use crate::widgets::{KeyDesc, KeyId, KeyMaps};

use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear};
use ratatui::Frame;

use super::models::SimulationSpec;
use super::{
    CommandInterpreter, FileExplorer, InstanceHierViewer, KeyMapsViewer, SignalsViewer, WaveViewer, TokioSender
};

pub struct Root {
    message_tx: Sender<RendererMessage>,
    request_tx: TokioSender<oombak_sim::Message>,
    signals_viewer: SignalsViewer,
    wave_viewer: WaveViewer,
    key_maps_viewer: KeyMapsViewer,
    instance_hier_viewer: Arc<RwLock<InstanceHierViewer>>,
    command_interpreter: Arc<RwLock<CommandInterpreter>>,
    file_explorer: Arc<RwLock<FileExplorer>>,
    focused_child: Option<Child>,
    simulation_spec: SimulationSpec,
    key_mappings: KeyMaps,
    show_key_maps: bool,
}

enum Child {
    CommandInterpreter,
    InstanceHierView,
    FileExplorer,
}

impl Root {
    pub fn new(
        message_tx: Sender<RendererMessage>,
        request_tx: TokioSender<oombak_sim::Message>,
        command_interpreter: Arc<RwLock<CommandInterpreter>>,
    ) -> Self {
        let simulation_spec = SimulationSpec::default();
        let key_mappings = Self::create_key_mappings();
        Self {
            message_tx: message_tx.clone(),
            request_tx: request_tx.clone(),
            wave_viewer: WaveViewer::default().simulation(simulation_spec.clone()),
            signals_viewer: SignalsViewer::default().simulation(simulation_spec.clone()),
            key_maps_viewer: KeyMapsViewer::new(key_mappings.clone()),
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
            key_mappings,
            show_key_maps: false,
        }
    }

    fn create_key_mappings() -> KeyMaps {
        HashMap::from([
            (KeyId::from('q'), KeyDesc::from("quit")),
            (KeyId::from('o'), KeyDesc::from("load sim file")),
            (KeyId::from('t'), KeyDesc::from("open terminal")),
            (KeyId::from('s'), KeyDesc::from("open probe editor")),
            (KeyId::from(':'), KeyDesc::from("open command line")),
            (KeyId::from(KeyCode::Up), KeyDesc::from("scroll up")),
            (KeyId::from('k'), KeyDesc::from("scroll up")),
            (KeyId::from(KeyCode::Down), KeyDesc::from("scroll down")),
            (KeyId::from('j'), KeyDesc::from("scroll down")),
        ])
        .into()
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
        if self.show_key_maps {
            self.render_key_maps_viewer(f, rect);
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
            KeyCode::F(1) => {
                self.key_maps_viewer.prev_page();
            }
            KeyCode::F(2) => {
                self.show_key_maps = !self.show_key_maps;
            }
            KeyCode::F(3) => {
                self.key_maps_viewer.next_page();
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

    fn handle_focus_gained(&mut self) -> HandleResult {
        self.notify_render();
        self.focused_child = None;
        HandleResult::Handled
    }

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>> {
        match self.focused_child {
            Some(Child::CommandInterpreter) => Some(self.command_interpreter.clone()),
            Some(Child::InstanceHierView) => Some(self.instance_hier_viewer.clone()),
            Some(Child::FileExplorer) => Some(self.file_explorer.clone()),
            None => None,
        }
    }

    fn get_key_mappings(&self) -> KeyMaps {
        let mut key_maps = match self.focused_child {
            Some(Child::CommandInterpreter) => {
                self.command_interpreter.read().unwrap().get_key_mappings()
            }
            Some(Child::InstanceHierView) => {
                self.instance_hier_viewer.read().unwrap().get_key_mappings()
            }
            Some(Child::FileExplorer) => self.file_explorer.read().unwrap().get_key_mappings(),
            None => self.key_mappings.clone(),
        };

        key_maps.insert(
            KeyId::from(KeyCode::F(2)),
            KeyDesc::from("toggle help").prio(-1),
        );
        key_maps
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
        let popup_area = Self::get_popup_area_centered_large(rect);
        let block = Block::new().borders(Borders::ALL);
        f.render_widget(Clear, popup_area);
        self.instance_hier_viewer
            .write()
            .unwrap()
            .render_with_block(f, popup_area, block);
    }

    fn render_file_explorer(&self, f: &mut Frame, rect: Rect) {
        let popup_area = Self::get_popup_area_centered_large(rect);
        let block = Block::new().borders(Borders::ALL);
        f.render_widget(Clear, popup_area);
        self.file_explorer
            .write()
            .unwrap()
            .render_with_block(f, popup_area, block);
    }

    fn render_key_maps_viewer(&mut self, f: &mut Frame, rect: Rect) {
        self.key_maps_viewer
            .set_key_maps(self.get_key_mappings().clone());
        let popup_area = Self::get_popup_area_bottom_right(rect);
        let block = Block::new()
            .borders(Borders::ALL)
            .title_top("Command Keys (F2)")
            .title_bottom(" ← F1 | F3 → ")
            .title_alignment(Alignment::Center);
        let inner = block.inner(popup_area);
        f.render_widget(Clear, popup_area);
        f.render_widget(block, popup_area);
        self.key_maps_viewer.render_mut(f, inner);
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
        let popup_area = Self::get_popup_area_centered_large(rect);
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

    fn get_popup_area_centered_large(rect: Rect) -> Rect {
        Self::get_popup_area_centered(rect, 3, 6)
    }

    fn get_popup_area_bottom_right(rect: Rect) -> Rect {
        let min_width = 40;
        let min_height = 13;
        let top_margin = 3.max(rect.height as i64 - min_height - 3);
        let left_margin = 6.max(rect.width as i64 - min_width - 6);
        Self::get_popup_area(rect, top_margin as u16, 6, 3, left_margin as u16)
    }

    fn get_popup_area_centered(rect: Rect, vert_margin: u16, hor_margin: u16) -> Rect {
        Self::get_popup_area(rect, vert_margin, hor_margin, vert_margin, hor_margin)
    }

    fn get_popup_area(
        rect: Rect,
        top_margin: u16,
        right_margin: u16,
        bottom_margin: u16,
        left_margin: u16,
    ) -> Rect {
        let chunks = Layout::vertical(vec![
            Constraint::Length(top_margin),
            Constraint::Min(0),
            Constraint::Length(bottom_margin),
        ])
        .split(rect);
        let chunks = Layout::horizontal(vec![
            Constraint::Length(left_margin),
            Constraint::Min(0),
            Constraint::Length(right_margin),
        ])
        .split(chunks[1]);
        chunks[1]
    }
}

#[async_trait]
impl simulator_request_dispatcher::Listener for Root {
    async fn on_receive_reponse(&mut self, response: &oombak_sim::Response) {
        if let Some(result) = response.result() {
            match result {
                oombak_sim::response::Results::CurrentTime(_) => self.request_simulation_result().await,
                oombak_sim::response::Results::LoadedDut(dut) => {
                    self.set_loaded_dut(dut);
                    self.reload_simulation().await;
                }
                oombak_sim::response::Results::SimulationResult(res) => self.update_simulation(res),
                oombak_sim::response::Results::Empty => (),
            }
        }
    }
}

impl Root {
    async fn request_simulation_result(&self) {
        self.request_tx
            .send(oombak_sim::Request::get_simulation_result()).await
            .unwrap();
    }

    fn set_loaded_dut(&mut self, loaded_dut: &oombak_sim::response::LoadedDut) {
        self.instance_hier_viewer
            .write()
            .unwrap()
            .set_loaded_dut(loaded_dut);
    }

    async fn reload_simulation(&mut self) {
        self.simulation_spec = SimulationSpec::default();
        self.set_simulation();
        self.request_simulation_result().await;
    }

    fn update_simulation(&mut self, simulation_result: &oombak_sim::response::SimulationResult) {
        if self.simulation_spec.is_empty() {
            self.simulation_spec = SimulationSpec::new(simulation_result);
        } else {
            self.simulation_spec.update(simulation_result);
        }
        self.set_simulation();
    }

    fn set_simulation(&mut self) {
        self.signals_viewer
            .set_simulation(self.simulation_spec.clone());
        self.wave_viewer
            .set_simulation(self.simulation_spec.clone());
    }
}
