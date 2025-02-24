use std::sync::{
    self,
    mpsc::{Receiver, Sender},
};

use super::{Thread, ThreadResult};

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
