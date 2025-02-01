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
    text: String,
    result_history: Vec<Result<String, String>>,
    state: State,
    mode: Mode,
}

#[derive(PartialEq)]
enum State {
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
            text: "".to_string(),
            result_history: vec![],
            state: State::NotActive,
            mode: Mode::Line,
        }
    }

    pub fn is_window_mode(&self) -> bool {
        self.mode == Mode::Window
    }

    pub fn set_line_mode(&mut self) {
        self.mode = Mode::Line;
        self.state = State::Active;
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
            Mode::Window => todo!(),
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
        let paragraph = match self.state {
            State::Active => {
                let text = format!(":{}", self.text);
                Paragraph::new(text).black().on_light_yellow()
            }
            State::NotActive => match self.result_history.last() {
                Some(Ok(res)) => Paragraph::new(res.clone()).green().on_black(),
                Some(Err(res)) => Paragraph::new(res.clone()).red().on_black(),
                _ => Paragraph::new("").on_black(),
            },
        };
        f.render_widget(paragraph, rect);
    }

    pub fn render_on_window(&self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        f.render_stateful_widget(Terminal::default(), rect, &mut TerminalState::default());
    }

    fn handle_key_event_line_mode(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Esc => {
                self.state = State::NotActive;
                self.text = "".to_string();
                return HandleResult::ReleaseFocus;
            }
            KeyCode::Enter => {
                self.execute_command();
                self.state = State::NotActive;
                self.text = "".to_string();
                return HandleResult::ReleaseFocus;
            }
            KeyCode::Char(c) if self.state == State::Active => {
                self.text += &format!("{c}");
            }
            KeyCode::Backspace if self.state == State::Active => {
                self.text.pop();
            }
            _ => (),
        };
        self.notify_render();
        HandleResult::Handled
    }

    fn execute_command(&mut self) {
        match interpreter::interpret(&self.text) {
            Ok(command) => {
                match command {
                    interpreter::Command::Run(x) => self.request(sim::Request::Run(x)),
                    interpreter::Command::Load(x) => self.request(sim::Request::Load(x)),
                    interpreter::Command::Set(sig_name, value) => {
                        self.request(sim::Request::SetSignal(sig_name, value))
                    }
                    interpreter::Command::Noop => return,
                }
                self.result_history
                    .push(Ok(format!("executed: {}", self.text)));
            }
            Err(message) => self.result_history.push(Err(message)),
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
        self.result_history.push(result);
        self.notify_render();
    }
}
