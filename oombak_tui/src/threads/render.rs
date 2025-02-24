use crate::component::Component;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use std::io::Stdout;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

use super::thread_group::ThreadGroup;
use super::util::any_to_string;
use super::{Thread, ThreadError, ThreadResult};

#[derive(PartialEq)]
pub enum Message {
    Quit,
    Render,
}

pub struct RendererThread {
    handle: Option<JoinHandle<ThreadResult>>,
    message_channel_tx: Sender<Message>,
}

impl RendererThread {
    pub fn new(
        root_component: Arc<RwLock<dyn Component>>,
        message_channel_tx: Sender<Message>,
        message_channel_rx: Receiver<Message>,
        mut terminal: Terminal<CrosstermBackend<Stdout>>,
        thread_group: &ThreadGroup,
    ) -> Self {
        let terminate_group_channel_tx = thread_group.get_terminate_group_channel_tx();

        let handle = thread::spawn(move || -> ThreadResult {
            let mut message = Message::Render;
            while message != Message::Quit {
                match terminal
                    .draw(|frame| root_component.write().unwrap().render(frame, frame.area()))
                {
                    Ok(_) => (),
                    Err(e) => {
                        let _ = terminate_group_channel_tx.send(());
                        return Err(ThreadError::Io(e));
                    }
                }
                message = message_channel_rx.recv().unwrap_or(Message::Quit);
            }

            let _ = terminate_group_channel_tx.send(());
            Ok(())
        });

        Self {
            handle: Some(handle),
            message_channel_tx,
        }
    }
}

impl Thread for RendererThread {
    fn terminate(&mut self) -> ThreadResult {
        if let Some(handle) = self.handle.take() {
            let _ = self.message_channel_tx.send(Message::Quit);
            match handle.join() {
                Err(e) => Err(ThreadError::Panic(any_to_string(&e))),
                Ok(res) => res,
            }
        } else {
            Ok(())
        }
    }
}
