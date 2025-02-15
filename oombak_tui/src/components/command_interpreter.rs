use std::sync::mpsc::Sender;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{style::Stylize, text::Line};

use crate::{
    backend::interpreter,
    component::{Component, HandleResult},
    render::Message,
    widgets::{CommandLine, Terminal, TerminalState},
};

use oombak_sim::sim;

pub struct CommandInterpreter {
    message_tx: Sender<Message>,
    request_tx: Sender<sim::Request>,
    terminal_state: TerminalState,
    line_state: LineState,
    mode: Mode,
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
    pub fn new(message_tx: Sender<Message>, request_tx: Sender<sim::Request>) -> Self {
        Self {
            message_tx,
            request_tx,
            terminal_state: TerminalState::default(),
            line_state: LineState::NotActive,
            mode: Mode::Line,
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

    fn try_propagate_event(&mut self, _event: &crossterm::event::Event) -> HandleResult {
        HandleResult::NotHandled
    }

    fn set_focus_to_self(&mut self) {}
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
                    Some(Ok(res)) => Line::from(&res[..]).green(),
                    Some(Err(res)) => Line::from(&res[..]).red(),
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
            _ => (),
        };
        self.notify_render();
        HandleResult::Handled
    }

    fn execute_command(&mut self) {
        let command_text = self.terminal_state.command_line_state().text();
        match interpreter::interpret(command_text) {
            Ok(command) => match command {
                interpreter::Command::Run(x) => {
                    self.request(sim::Request::Run(x));
                    self.terminal_state
                        .append_output_history(Ok(format!("executed: {command_text}")));
                }
                interpreter::Command::Load(x) => {
                    self.request(sim::Request::Load(x));
                    self.terminal_state
                        .append_output_history(Ok(format!("executed: {command_text}")));
                }
                interpreter::Command::Set(sig_name, value) => {
                    self.request(sim::Request::SetSignal(sig_name, value));
                    self.terminal_state
                        .append_output_history(Ok(format!("executed: {command_text}")));
                }
                interpreter::Command::Help => {
                    self.terminal_state
                        .append_output_history(Ok(interpreter::help().to_string()));
                }
                interpreter::Command::Quit => self.notify_quit(),
                interpreter::Command::Noop => (),
            },
            Err(message) => self.terminal_state.append_output_history(Err(message)),
        }
    }

    fn request(&self, request: sim::Request) {
        self.request_tx.send(request).unwrap();
    }

    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
    }

    fn notify_quit(&self) {
        self.message_tx.send(Message::Quit).unwrap();
    }
}

impl sim::Listener for CommandInterpreter {
    fn on_receive_reponse(&mut self, response: &sim::Response) {
        let result = match response {
            sim::Response::RunResult(Ok(curr_time)) => {
                Ok(format!("run: current time = {curr_time}"))
            }
            sim::Response::SetSignalResult(Ok(())) => Ok("set: success".to_string()),
            sim::Response::LoadResult(Ok(_)) => Ok("load: success".to_string()),
            sim::Response::RunResult(Err(e)) => Err(format!("run: {e}")),
            sim::Response::SetSignalResult(Err(e)) => Err(format!("set: {e}")),
            sim::Response::LoadResult(Err(e)) => Err(format!("load: {e}")),
            sim::Response::ModifyProbedPointsResult(Err(e)) => {
                Err(format!("modify probe points: {e}"))
            }
            _ => return,
        };
        self.terminal_state.append_output_history(result);
        self.notify_render();
    }
}
