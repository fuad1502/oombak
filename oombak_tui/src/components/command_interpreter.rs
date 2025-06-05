use std::{collections::HashMap, sync::mpsc::Sender};

use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{style::Stylize, text::Line};

use crate::{
    backend::interpreter,
    component::{Component, HandleResult},
    styles::terminal::{ERROR_OUTPUT_STYLE, NORMAL_OUTPUT_STYLE, NOTIFICATION_OUTPUT_STYLE},
    threads::{simulator_request_dispatcher, RendererMessage},
    widgets::{CommandLine, KeyDesc, KeyId, KeyMaps, Terminal, TerminalOutput, TerminalState},
};

use super::TokioSender;

pub struct CommandInterpreter {
    message_tx: Sender<RendererMessage>,
    request_tx: TokioSender<oombak_sim::Message>,
    terminal_state: TerminalState,
    line_state: LineState,
    mode: Mode,
    key_mappings: KeyMaps,
}

#[derive(PartialEq)]
enum LineState {
    Active,
    NotActive,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Mode {
    Line,
    Window,
}

impl CommandInterpreter {
    pub fn new(
        message_tx: Sender<RendererMessage>,
        request_tx: TokioSender<oombak_sim::Message>,
    ) -> Self {
        let key_mappings = Self::create_key_mappings();
        Self {
            message_tx,
            request_tx,
            terminal_state: TerminalState::default(),
            line_state: LineState::NotActive,
            mode: Mode::Line,
            key_mappings,
        }
    }

    pub fn is_window_mode(&self) -> bool {
        self.mode == Mode::Window
    }

    pub fn set_line_mode(&mut self) {
        self.mode = Mode::Line;
        self.line_state = LineState::Active;
    }

    pub fn set_window_mode(&mut self) {
        self.mode = Mode::Window;
    }

    fn create_key_mappings() -> KeyMaps {
        HashMap::from([
            (KeyId::from(KeyCode::Esc), KeyDesc::from("close window")),
            (
                KeyId::from((KeyCode::Char('d'), KeyModifiers::CONTROL)),
                KeyDesc::from("close window"),
            ),
            (
                KeyId::from(KeyCode::Enter),
                KeyDesc::from("execute command"),
            ),
        ])
        .into()
    }
}

impl Component for CommandInterpreter {
    fn render(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        self.render_on_window(f, rect);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match self.mode {
            Mode::Line => self.handle_key_event_line_mode(key_event),
            Mode::Window => self.handle_key_event_window_mode(key_event),
        }
    }

    fn handle_resize_event(&mut self, _columns: u16, _rows: u16) -> HandleResult {
        self.notify_render();
        HandleResult::Handled
    }

    fn handle_focus_gained(&mut self) -> HandleResult {
        HandleResult::Handled
    }

    fn get_focused_child(&self) -> Option<std::sync::Arc<std::sync::RwLock<dyn Component>>> {
        None
    }

    fn get_key_mappings(&self) -> KeyMaps {
        self.key_mappings.clone()
    }
}

impl CommandInterpreter {
    pub fn render_on_line(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        match self.line_state {
            LineState::Active => {
                f.render_stateful_widget(
                    CommandLine::default(),
                    rect,
                    self.terminal_state.command_line_state_mut(),
                );
            }
            LineState::NotActive => {
                let line = match self.terminal_state.output_history().last() {
                    Some(TerminalOutput::Normal(res)) => {
                        Line::from(&res[..]).style(NORMAL_OUTPUT_STYLE)
                    }
                    Some(TerminalOutput::Notification(res)) => {
                        Line::from(&res[..]).style(NOTIFICATION_OUTPUT_STYLE)
                    }
                    Some(TerminalOutput::Error(res)) => {
                        Line::from(&res[..]).style(ERROR_OUTPUT_STYLE)
                    }
                    _ => Line::from(" "),
                };
                f.render_widget(line.on_dark_gray(), rect);
            }
        };
    }

    pub fn render_on_window(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        f.render_stateful_widget(Terminal::default(), rect, &mut self.terminal_state);
    }

    fn handle_key_event_line_mode(&mut self, key_event: &KeyEvent) -> HandleResult {
        let mut handle_result = self.handle_key_event_window_mode(key_event);
        if key_event.code == KeyCode::Enter {
            handle_result = HandleResult::ReleaseFocus;
        }
        if handle_result == HandleResult::ReleaseFocus {
            self.line_state = LineState::NotActive;
        }
        handle_result
    }

    fn handle_key_event_window_mode(&mut self, key_event: &KeyEvent) -> HandleResult {
        let command_line_state = self.terminal_state.command_line_state_mut();
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Esc, _) => {
                command_line_state.clear();
                return HandleResult::ReleaseFocus;
            }
            (KeyCode::Char('d'), modifier) if modifier.contains(KeyModifiers::CONTROL) => {
                command_line_state.clear();
                return HandleResult::ReleaseFocus;
            }
            (KeyCode::Enter, _) => {
                self.execute_command();
                self.terminal_state.command_line_state_mut().clear();
            }
            (KeyCode::Char(c), modifier) if modifier.is_empty() => {
                command_line_state.put(c);
            }
            (KeyCode::Backspace, _) => {
                command_line_state.backspace();
            }
            (KeyCode::Right, _) => {
                command_line_state.move_cursor_right();
            }
            (KeyCode::Left, _) => {
                command_line_state.move_cursor_left();
            }
            (KeyCode::F(_), _) => {
                return HandleResult::NotHandled;
            }
            _ => (),
        };
        self.notify_render();
        HandleResult::Handled
    }

    fn execute_command(&mut self) {
        let command_text = self.terminal_state.command_line_state().text();
        match interpreter::interpret(command_text) {
            Ok(command) => match command {
                interpreter::Command::Run(duration) => {
                    let request = oombak_sim::Request::run(duration);
                    self.request_tx.blocking_send(request).unwrap();
                    self.terminal_state
                        .append_output_history(TerminalOutput::Normal(command_text.to_string()));
                }
                interpreter::Command::Load(sv_path) => {
                    let request = oombak_sim::Request::load(sv_path);
                    self.request_tx.blocking_send(request).unwrap();
                    self.terminal_state
                        .append_output_history(TerminalOutput::Normal(command_text.to_string()));
                }
                interpreter::Command::Set(signal_name, value) => {
                    let request = oombak_sim::Request::set_signal(signal_name, value);
                    self.request_tx.blocking_send(request).unwrap();
                    self.terminal_state
                        .append_output_history(TerminalOutput::Normal(command_text.to_string()));
                }
                interpreter::Command::Help => {
                    self.terminal_state
                        .append_output_history(TerminalOutput::Normal(
                            interpreter::help().to_string(),
                        ));
                }
                interpreter::Command::Quit => self.notify_quit(),
                interpreter::Command::Noop => (),
            },
            Err(message) => self
                .terminal_state
                .append_output_history(TerminalOutput::Error(message)),
        }
    }

    fn notify_render(&self) {
        self.message_tx.send(RendererMessage::Render).unwrap();
    }

    fn notify_quit(&self) {
        self.message_tx.send(RendererMessage::Quit).unwrap();
    }
}

#[async_trait]
impl simulator_request_dispatcher::Listener for CommandInterpreter {
    async fn on_receive_reponse(&mut self, response: &oombak_sim::Response) {
        let id = response.id;
        let result = match &response.payload {
            oombak_sim::response::Payload::Result(_) => {
                TerminalOutput::Normal(format!("[ID: {:x}] Finished", id))
            }
            oombak_sim::response::Payload::Notification(notification) => {
                TerminalOutput::Notification(format!("[ID: {:x}] {}", id, notification))
            }
            oombak_sim::response::Payload::Error(e) => {
                TerminalOutput::Error(format!("[ID: {:x}] Error: {}", id, e))
            }
        };
        self.terminal_state.append_output_history(result);
        self.notify_render();
    }
}
