use std::sync::mpsc::Sender;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{style::Stylize, widgets::Paragraph};

use crate::{backend::interpreter, component::Component, render::Message};

use oombak_sim::sim;

pub struct CommandLine {
    message_tx: Sender<Message>,
    request_tx: Sender<sim::Request>,
    text: String,
    result_history: Vec<Result<String, String>>,
    state: State,
}

#[derive(PartialEq)]
enum State {
    Active,
    NotActive,
}

impl CommandLine {
    pub fn new(message_tx: Sender<Message>, request_tx: Sender<sim::Request>) -> Self {
        Self {
            message_tx,
            request_tx,
            text: "".to_string(),
            result_history: vec![],
            state: State::NotActive,
        }
    }
}

impl Component for CommandLine {
    fn render(&self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        let paragraph = match self.state {
            State::Active => Paragraph::new(self.text.clone()).black().on_light_yellow(),
            State::NotActive => match self.result_history.last() {
                Some(Ok(res)) => Paragraph::new(res.clone()).green().on_black(),
                Some(Err(res)) => Paragraph::new(res.clone()).red().on_black(),
                _ => Paragraph::new("").on_black(),
            },
        };
        f.render_widget(paragraph, rect);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Esc => {
                self.state = State::NotActive;
                self.notify_render();
                return false;
            }
            KeyCode::Enter => {
                self.state = State::NotActive;
                self.execute_command();
                self.notify_render();
                return false;
            }
            KeyCode::Char(':') => {
                self.state = State::Active;
                self.text = ":".to_string();
            }
            KeyCode::Char(c) if self.state == State::Active => {
                self.text += &format!("{c}");
            }
            KeyCode::Backspace if self.state == State::Active && self.text.len() > 1 => {
                self.text.pop();
            }
            _ => (),
        };
        self.notify_render();
        true
    }

    fn set_focus(&mut self) {}

    fn get_focused_child(&mut self) -> Option<&mut dyn Component> {
        None
    }
}

impl CommandLine {
    fn execute_command(&mut self) {
        let command_string = &self.text[1..];
        match interpreter::interpret(command_string) {
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
                    .push(Ok(format!("executed: {command_string}")));
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

impl sim::Listener for CommandLine {
    fn on_receive_reponse(&mut self, response: &sim::Response) {
        let result = match response {
            sim::Response::RunResult(Ok(curr_time)) => {
                Ok(format!("run: current time = {curr_time}"))
            }
            sim::Response::SetSignalResult(Ok(())) => Ok("set: success".to_string()),
            sim::Response::LoadResult(Ok(())) => Ok("load: success".to_string()),
            sim::Response::RunResult(Err(e)) => Err(format!("run: {e}")),
            sim::Response::SetSignalResult(Err(e)) => Err(format!("set: {e}")),
            sim::Response::LoadResult(Err(e)) => Err(format!("load: {e}")),
            sim::Response::SimulationResult(_) => return,
        };
        self.result_history.push(result);
        self.notify_render();
    }
}
