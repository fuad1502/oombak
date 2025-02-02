use std::sync::mpsc::Sender;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{style::Stylize, widgets::Paragraph};

use crate::{
    backend::interpreter,
    component::{Component, HandleResult},
    render::Message,
    widgets::{Terminal, TerminalState},
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
    fn render(&self, _f: &mut ratatui::Frame, _rect: ratatui::prelude::Rect) {}

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
    pub fn render_on_line(&self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let paragraph = match self.line_state {
            LineState::Active => {
                let text = format!(":{}", self.terminal_state.command_line());
                Paragraph::new(text).black().on_light_yellow()
            }
            LineState::NotActive => match self.terminal_state.output_history().last() {
                Some(Ok(res)) => Paragraph::new(res.clone()).green().on_black(),
                Some(Err(res)) => Paragraph::new(res.clone()).red().on_black(),
                _ => Paragraph::new("").on_black(),
            },
        };
        f.render_widget(paragraph, rect);
    }

    pub fn render_on_window(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        f.render_stateful_widget(Terminal::default(), rect, &mut self.terminal_state);
    }

    fn handle_key_event_line_mode(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Esc => {
                self.line_state = LineState::NotActive;
                self.terminal_state.clear_command_line();
                return HandleResult::ReleaseFocus;
            }
            KeyCode::Enter => {
                self.execute_command();
                self.line_state = LineState::NotActive;
                self.terminal_state.clear_command_line();
                return HandleResult::ReleaseFocus;
            }
            KeyCode::Char(c) if self.line_state == LineState::Active => {
                self.terminal_state.put(c);
            }
            KeyCode::Backspace if self.line_state == LineState::Active => {
                self.terminal_state.backspace();
            }
            _ => (),
        };
        self.notify_render();
        HandleResult::Handled
    }

    fn handle_key_event_window_mode(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Esc => {
                self.terminal_state.clear_command_line();
                return HandleResult::ReleaseFocus;
            }
            KeyCode::Enter => {
                if let Some(result) = self.execute_builtin_command() {
                    return result;
                }
                self.execute_command();
                self.terminal_state.clear_command_line();
            }
            KeyCode::Char(c) => {
                self.terminal_state.put(c);
            }
            KeyCode::Backspace => {
                self.terminal_state.backspace();
            }
            KeyCode::Right => {
                self.terminal_state.move_cursor_right();
            }
            KeyCode::Left => {
                self.terminal_state.move_cursor_left();
            }
            _ => (),
        };
        self.notify_render();
        HandleResult::Handled
    }

    fn execute_builtin_command(&mut self) -> Option<HandleResult> {
        match self.terminal_state.command_line() {
            "quit" => {
                self.terminal_state.clear_command_line();
                Some(HandleResult::ReleaseFocus)
            }
            "help" => todo!(),
            _ => None,
        }
    }

    fn execute_command(&mut self) {
        let command_text = self.terminal_state.command_line();
        match interpreter::interpret(command_text) {
            Ok(command) => {
                match command {
                    interpreter::Command::Run(x) => self.request(sim::Request::Run(x)),
                    interpreter::Command::Load(x) => self.request(sim::Request::Load(x)),
                    interpreter::Command::Set(sig_name, value) => {
                        self.request(sim::Request::SetSignal(sig_name, value))
                    }
                    interpreter::Command::Noop => return,
                }
                self.terminal_state
                    .append_output_history(Ok(format!("executed: {command_text}")));
            }
            Err(message) => self.terminal_state.append_output_history(Err(message)),
        }
    }

    fn request(&self, request: sim::Request) {
        self.request_tx.send(request).unwrap();
    }

    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
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
