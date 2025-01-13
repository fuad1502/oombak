use crate::{component::Component, error::OombakResult};

use crossterm::event::Event;

use std::{
    sync::{mpsc, Arc, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

static LISTENERS: RwLock<Vec<Arc<RwLock<dyn Component>>>> = RwLock::new(vec![]);

pub fn register_event_listener(listener: Arc<RwLock<dyn Component>>) {
    LISTENERS.write().unwrap().push(listener);
}

pub fn spawn_event_loop() -> (JoinHandle<OombakResult<()>>, mpsc::Sender<()>) {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let event_handle = thread::spawn(move || -> OombakResult<()> {
        while stop_rx.try_recv().is_err() {
            if crossterm::event::poll(Duration::from_millis(500))? {
                notify_event_listeners(&crossterm::event::read()?);
            }
        }
        Ok(())
    });
    (event_handle, stop_tx)
}

fn notify_event_listeners(event: &Event) {
    let listeners = LISTENERS.read().unwrap();
    for listener in listeners.iter() {
        listener.write().unwrap().handle_event(event);
    }
}
