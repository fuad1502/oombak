use crate::{component::Component, error::OmbakResult};

use crossterm::event::Event;

use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

static LISTENERS: Mutex<Vec<Arc<Mutex<dyn Component + Send>>>> = Mutex::new(vec![]);

pub fn register_event_listener(listener: Arc<Mutex<dyn Component + Send>>) {
    LISTENERS.lock().unwrap().push(listener);
}

pub fn spawn_event_loop() -> (JoinHandle<OmbakResult<()>>, mpsc::Sender<()>) {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let event_handle = thread::spawn(move || -> OmbakResult<()> {
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
    let listeners = LISTENERS.lock().unwrap();
    for listener in listeners.iter() {
        listener.lock().unwrap().handle_event(event);
    }
}
