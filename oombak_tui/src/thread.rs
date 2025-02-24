use thiserror::Error;

use std::{
    any::Any,
    panic::{self, PanicHookInfo},
    sync::{
        self,
        mpsc::{Receiver, Sender},
    },
};

pub type ThreadResult = Result<(), ThreadError>;

#[derive(Error, Debug)]
pub enum ThreadError {
    #[error("thread panicked: {}", _0)]
    Panic(String),
    #[error("IO error: {}", _0)]
    Io(std::io::Error),
}

pub trait Thread {
    fn terminate(&mut self) -> ThreadResult;
}

pub struct ThreadGroup {
    terminate_channel_rx: Receiver<()>,
    terminate_channel_tx: Sender<()>,
    threads: Vec<Box<dyn Thread>>,
}

impl ThreadGroup {
    pub fn new() -> Self {
        let (terminate_channel_tx, terminate_channel_rx) = sync::mpsc::channel();
        Self {
            terminate_channel_rx,
            terminate_channel_tx,
            threads: vec![],
        }
    }

    pub fn add_thread(&mut self, thread: Box<dyn Thread>) {
        self.threads.push(thread);
    }

    pub fn get_terminate_group_channel_tx(&self) -> Sender<()> {
        self.terminate_channel_tx.clone()
    }

    pub fn join(self) -> Vec<ThreadResult> {
        let mut results = vec![];
        let _ = self.terminate_channel_rx.recv();
        for mut thread in self.threads {
            results.push(thread.terminate());
        }
        results
    }
}

impl Default for ThreadGroup {
    fn default() -> Self {
        Self::new()
    }
}

pub fn setup_terminate_group_panic_hook(terminate_group_channel_tx: Sender<()>) {
    let original_hook = panic::take_hook();
    let panic_handler = move |hook_info: &PanicHookInfo| {
        let _ = terminate_group_channel_tx.send(());
        original_hook(hook_info);
    };
    panic::set_hook(Box::new(panic_handler));
}

pub fn any_to_string(any: &Box<dyn Any + Send>) -> String {
    if let Some(message) = any.downcast_ref::<&'static str>() {
        message.to_string()
    } else if let Some(message) = any.downcast_ref::<String>() {
        message.clone()
    } else {
        format!("{:?}", any)
    }
}
