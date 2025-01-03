use crate::component::Component;
use crate::error::OmbakResult;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use std::io::Stdout;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

#[derive(PartialEq)]
pub enum Message {
    Quit,
    Render,
}

pub fn spawn_renderer(
    root: Arc<RwLock<dyn Component>>,
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
    message_rx: Receiver<Message>,
) -> JoinHandle<OmbakResult<()>> {
    thread::spawn(move || -> OmbakResult<()> {
        let mut message = Message::Render;
        while message != Message::Quit {
            terminal.draw(|frame| root.write().unwrap().render(frame, frame.area()))?;
            message = message_rx.recv()?;
        }
        Ok(())
    })
}
