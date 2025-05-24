use oombak_sim::LocalSimulator;
use oombak_tui::{
    components,
    threads::{
        setup_terminate_group_panic_hook, EventThread, RendererThread, SimulatorRequestDispatcher,
        ThreadGroup,
    },
    tui,
};
use std::sync::{mpsc, Arc, RwLock};

fn main() {
    let terminal = tui::init_terminal().unwrap();
    let simulator = LocalSimulator::default();
    let simulator_request_dispatcher = SimulatorRequestDispatcher::new(Arc::new(simulator));
    let (message_channel_tx, message_channel_rx) = mpsc::channel();

    let command_interpreter = Arc::new(RwLock::new(components::CommandInterpreter::new(
        message_channel_tx.clone(),
        simulator_request_dispatcher.channel(),
    )));

    let root = Arc::new(RwLock::new(components::Root::new(
        message_channel_tx.clone(),
        simulator_request_dispatcher.channel(),
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

    simulator_request_dispatcher.register(command_interpreter);
    simulator_request_dispatcher.register(root.clone());
    event_thread.register_event_listener(root);

    thread_group.add_thread(Box::new(event_thread));
    thread_group.add_thread(Box::new(renderer_thread));
    thread_group.add_thread(Box::new(simulator_request_dispatcher));
    setup_terminate_group_panic_hook(&thread_group);
    let res = thread_group.join();

    tui::restore_terminal().unwrap();

    eprintln!("Thread termination status:");
    for res in res {
        match res {
            Ok(_) => eprintln!("> Thread terminated normally"),
            Err(e) => eprintln!("> Thread terminated abnormally: {e}"),
        }
    }
}
