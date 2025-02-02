use oombak_sim::sim;
use oombak_tui::{components, event, render, tui};
use std::sync::{mpsc, Arc, RwLock};

fn main() {
    let terminal = tui::init_terminal().unwrap();

    let (message_tx, message_rx) = mpsc::channel();
    let mut simulator = sim::Simulator::new().unwrap();

    let command_interpreter = Arc::new(RwLock::new(components::CommandInterpreter::new(
        message_tx.clone(),
        simulator.get_request_channel(),
    )));
    simulator.register_listener(command_interpreter.clone());

    let root = Arc::new(RwLock::new(components::Root::new(
        message_tx,
        simulator.get_request_channel(),
        command_interpreter,
    )));
    simulator.register_listener(root.clone());
    event::register_event_listener(root.clone());

    event::spawn_event_loop();
    render::spawn_renderer(root, terminal, message_rx)
        .join()
        .unwrap()
        .unwrap();

    simulator
        .get_request_channel()
        .send(sim::Request::Terminate)
        .unwrap();
    tui::restore_terminal().unwrap();
}
