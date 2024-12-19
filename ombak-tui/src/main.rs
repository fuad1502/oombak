use ombak_tui::{components, event, render, tui};
use std::sync::{mpsc, Arc, Mutex};

fn main() {
    let terminal = tui::init_terminal().unwrap();
    let (message_tx, message_rx) = mpsc::channel();
    let root = components::Root::new(message_tx);
    let listener = Arc::new(Mutex::new(root));
    let component = Arc::clone(&listener);

    event::register_event_listener(listener);
    event::spawn_event_loop();
    let _ = render::spawn_renderer(component, terminal, message_rx).join();

    tui::restore_terminal().unwrap();
}
