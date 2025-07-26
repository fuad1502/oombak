use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::component::{Component, HandleResult};
use crate::threads::{simulator_request_dispatcher, RendererMessage};
use crate::utils;
use crate::widgets::{KeyDesc, KeyId, KeyMaps};

use crossterm::event::{KeyCode, KeyEvent};

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Clear};
use ratatui::Frame;

use super::models::SimulationSpec;
use super::signal_properties_editor::SignalPropertiesEditor;
use super::{
    CommandInterpreter, FileExplorer, InstanceHierViewer, KeyMapsViewer, SignalsViewer,
    TokioSender, WaveViewer,
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
    signal_properties_editor: Arc<RwLock<SignalPropertiesEditor>>,
    focused_child: Option<Child>,
    simulation_spec: Arc<RwLock<SimulationSpec>>,
    key_mappings: KeyMaps,
    show_key_maps: bool,
}

enum Child {
    CommandInterpreter,
    InstanceHierView,
    FileExplorer,
    SignalPropertiesEditor,
}

impl Root {
    pub fn new(
        message_tx: Sender<RendererMessage>,
        request_tx: TokioSender<oombak_sim::Message>,
        command_interpreter: Arc<RwLock<CommandInterpreter>>,
    ) -> Self {
        let simulation_spec = Arc::new(RwLock::new(SimulationSpec::default()));
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
            signal_properties_editor: Arc::new(RwLock::new(SignalPropertiesEditor::new(
                simulation_spec.clone(),
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
        if matches!(self.focused_child, Some(Child::SignalPropertiesEditor)) {
            self.render_signal_properties_editor(f, rect);
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
                self.wave_viewer.zoom_in();
                self.update_signal_viewer_highlight();
            }
            KeyCode::Char('-') | KeyCode::Char('x') => {
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
            KeyCode::Enter => {
                if let Some(signal_name) = self.signals_viewer.selected_signal_name() {
                    self.signal_properties_editor
                        .write()
                        .unwrap()
                        .set_signal_name(&signal_name);
                    self.focused_child = Some(Child::SignalPropertiesEditor);
                }
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
            Some(Child::SignalPropertiesEditor) => Some(self.signal_properties_editor.clone()),
            None => None,
        }
    }

    fn get_key_mappings(&self) -> KeyMaps {
        let mut key_maps = match self.get_focused_child() {
            Some(child) => child.read().unwrap().get_key_mappings(),
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
        // SignalsViewer and WaveViewer render area height must be in sync to allow synchronized
        // scrolling. Right now, rendering WaveViewer implicitly render TimeBar on the bottom 3
        // lines, therefore, we cut SignalsViewer render area height here.
        // TODO: refactor
        let areas = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(rect);
        self.signals_viewer.render_mut(f, areas[0]);
    }

    fn render_wave_viewer(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::new().borders(Borders::LEFT);
        f.render_widget(block, rect);
        // WaveViewer is rendered overlapping with the left border to get a collapsed border look
        self.wave_viewer.render_mut(f, rect);
    }

    fn render_instance_hier_viewer(&self, f: &mut Frame, rect: Rect) {
        let popup_area = Self::get_popup_area_centered_large(rect);
        let block = Block::bordered().border_type(BorderType::Rounded);
        f.render_widget(Clear, popup_area);
        self.instance_hier_viewer
            .write()
            .unwrap()
            .render_with_block(f, popup_area, block);
    }

    fn render_file_explorer(&self, f: &mut Frame, rect: Rect) {
        let popup_area = Self::get_popup_area_centered_large(rect);
        let block = Block::bordered().border_type(BorderType::Rounded);
        f.render_widget(Clear, popup_area);
        self.file_explorer
            .write()
            .unwrap()
            .render_with_block(f, popup_area, block);
    }

    fn render_signal_properties_editor(&self, f: &mut Frame, rect: Rect) {
        let popup_area = Self::get_popup_area_centered_large(rect);
        self.signal_properties_editor
            .write()
            .unwrap()
            .render(f, popup_area);
    }

    fn render_key_maps_viewer(&mut self, f: &mut Frame, rect: Rect) {
        self.key_maps_viewer
            .set_key_maps(self.get_key_mappings().clone());
        let popup_area = utils::layout::get_popup_area_bottom_right(rect);
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
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
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
        let width = rect.width - 12;
        let height = rect.height - 6;
        utils::layout::get_popup_area_centered(rect, width, height)
    }
}

impl simulator_request_dispatcher::Listener for Root {
    fn on_receive_reponse(&mut self, response: &oombak_sim::Response) {
        if let Some(result) = response.result() {
            match result {
                oombak_sim::response::Results::CurrentTime(_) => self.request_simulation_result(),
                oombak_sim::response::Results::LoadedDut(dut) => {
                    self.set_loaded_dut(dut);
                    self.reset_simulation_spec();
                }
                oombak_sim::response::Results::SimulationResult(res) => {
                    self.update_simulation_spec(res);
                }
                oombak_sim::response::Results::Empty => (),
            }
        }
    }
}

impl Root {
    fn set_loaded_dut(&mut self, loaded_dut: &oombak_sim::response::LoadedDut) {
        self.instance_hier_viewer
            .write()
            .unwrap()
            .set_loaded_dut(loaded_dut);
        self.signal_properties_editor
            .write()
            .unwrap()
            .set_loaded_dut(loaded_dut);
    }

    fn reset_simulation_spec(&mut self) {
        self.simulation_spec_mut().reset();
        self.reload_viewers();
        self.request_simulation_result();
    }

    fn request_simulation_result(&self) {
        self.request_tx
            .blocking_send(oombak_sim::Request::get_simulation_result())
            .unwrap();
    }

    fn update_simulation_spec(
        &mut self,
        simulation_result: &oombak_sim::response::SimulationResult,
    ) {
        if self.simulation_spec().is_empty() {
            self.simulation_spec_mut().reset_with(simulation_result);
        } else {
            self.simulation_spec_mut().update_with(simulation_result);
        }
        self.reload_viewers();
    }

    fn reload_viewers(&mut self) {
        self.signals_viewer.reload();
        self.wave_viewer.reload();
    }

    fn simulation_spec(&self) -> RwLockReadGuard<'_, SimulationSpec> {
        self.simulation_spec.read().unwrap()
    }

    fn simulation_spec_mut(&mut self) -> RwLockWriteGuard<'_, SimulationSpec> {
        self.simulation_spec.write().unwrap()
    }
}
