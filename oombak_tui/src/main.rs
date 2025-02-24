use oombak_sim::sim;
use oombak_tui::{
    components,
    event::EventThread,
    render::RendererThread,
    thread::{setup_terminate_group_panic_hook, ThreadGroup},
    tui,
};
use std::sync::{mpsc, Arc, RwLock};

fn main() {
    let terminal = tui::init_terminal().unwrap();
    let mut simulator = sim::Simulator::new().unwrap();
    let (message_channel_tx, message_channel_rx) = mpsc::channel();

    let command_interpreter = Arc::new(RwLock::new(components::CommandInterpreter::new(
        message_channel_tx.clone(),
        simulator.get_request_channel(),
    )));

    let root = Arc::new(RwLock::new(components::Root::new(
        message_channel_tx.clone(),
        simulator.get_request_channel(),
        command_interpreter.clone(),
    )));

    let mut thread_group = ThreadGroup::new();
    let mut event_thread = EventThread::new(&thread_group);
    let renderer_thread = RendererThread::new(
        root.clone(),
        message_channel_tx,
        message_channel_rx,
        terminal,
        &thread_group,
    );

    simulator.register_listener(command_interpreter);
    simulator.register_listener(root.clone());
    event_thread.register_event_listener(root);

    thread_group.add_thread(Box::new(event_thread));
    thread_group.add_thread(Box::new(renderer_thread));
    setup_terminate_group_panic_hook(thread_group.get_terminate_group_channel_tx());
    let res = thread_group.join();

    simulator
        .get_request_channel()
        .send(sim::Request::Terminate)
        .unwrap();
    tui::restore_terminal().unwrap();

    eprintln!("Thread termination status:");
    for res in res {
        match res {
            Ok(_) => eprintln!("> Thread terminated normally"),
            Err(e) => eprintln!("> Thread terminated abnormally: {e}"),
        }
    }
}
