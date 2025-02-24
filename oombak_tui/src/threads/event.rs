use crate::component::Component;

use crossterm::event::Event;

use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use super::{thread_group::ThreadGroup, util::any_to_string, Thread, ThreadError, ThreadResult};

pub struct EventThread {
    handle: Option<JoinHandle<ThreadResult>>,
    stop_channel_tx: Sender<()>,
    listeners: Arc<RwLock<Vec<SharedComponent>>>,
}

type SharedComponent = Arc<RwLock<dyn Component>>;

impl EventThread {
    pub fn new(thread_group: &ThreadGroup) -> Self {
        let (stop_channel_tx, stop_channel_rx) = mpsc::channel::<()>();
        let terminate_group_channel_tx = thread_group.get_terminate_group_channel_tx();
        let listeners = Arc::new(RwLock::new(vec![]));
        let listeners_clone = listeners.clone();

        let handle = thread::spawn(move || -> ThreadResult {
            while stop_channel_rx.try_recv().is_err() {
                match crossterm::event::poll(Duration::from_millis(500)) {
                    Err(e) => {
                        let _ = terminate_group_channel_tx.send(());
                        return Err(ThreadError::Io(e));
                    }
                    Ok(false) => (),
                    Ok(true) => match crossterm::event::read() {
                        Ok(event) => Self::notify_event_listeners(&listeners_clone, &event),
                        Err(e) => {
                            let _ = terminate_group_channel_tx.send(());
                            return Err(ThreadError::Io(e));
                        }
                    },
                }
            }
            Ok(())
        });

        EventThread {
            handle: Some(handle),
            stop_channel_tx,
            listeners,
        }
    }

    pub fn register_event_listener(&mut self, listener: SharedComponent) {
        self.listeners.write().unwrap().push(listener);
    }

    fn notify_event_listeners(listeners: &Arc<RwLock<Vec<SharedComponent>>>, event: &Event) {
        for listener in listeners.read().unwrap().iter() {
            listener.write().unwrap().handle_event(event);
        }
    }
}

impl Thread for EventThread {
    fn terminate(&mut self) -> ThreadResult {
        if let Some(handle) = self.handle.take() {
            let _ = self.stop_channel_tx.send(());
            match handle.join() {
                Err(e) => Err(ThreadError::Panic(any_to_string(&e))),
                Ok(res) => res,
            }
        } else {
            Ok(())
        }
    }
}
