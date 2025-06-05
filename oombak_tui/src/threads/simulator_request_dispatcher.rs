mod listener;

use std::{
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use oombak_sim::{Message, Simulator};
use tokio::{
    runtime::Builder,
    sync::mpsc::{self, Receiver, Sender},
};

use super::{util::any_to_string, Thread, ThreadError, ThreadResult};

pub use listener::{Listener, Listeners};

pub struct SimulatorRequestDispatcher {
    channel: Sender<Message>,
    listeners: Arc<RwLock<Listeners>>,
    handle: Option<JoinHandle<()>>,
}

impl SimulatorRequestDispatcher {
    pub fn new(simulator: Arc<dyn Simulator>) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let rt = Builder::new_current_thread().build().unwrap();
        rt.block_on(simulator.set_channel(tx.clone()));
        let listeners = Arc::new(RwLock::new(Vec::new()));
        let cloned_listeners = listeners.clone();
        let handle = thread::spawn(move || Self::thread(simulator, rx, cloned_listeners));
        Self {
            channel: tx,
            listeners,
            handle: Some(handle),
        }
    }

    pub fn channel(&self) -> Sender<Message> {
        self.channel.clone()
    }

    pub fn register(&self, listener: Arc<RwLock<dyn Listener>>) {
        self.listeners.write().unwrap().push(listener);
    }

    #[tokio::main(flavor = "current_thread")]
    async fn thread(
        simulator: Arc<dyn Simulator>,
        mut rx: Receiver<Message>,
        listeners: Arc<RwLock<Listeners>>,
    ) {
        while let Some(message) = rx.recv().await {
            match message {
                Message::Request(request)
                    if request.payload == oombak_sim::request::Payload::Terminate =>
                {
                    return
                }
                Message::Request(request) => {
                    Self::notify_request_dispatched(&request, listeners.clone()).await;
                    let sim = simulator.clone();
                    tokio::spawn(async move { sim.serve(&request).await });
                }
                Message::Response(response) => Self::notify(response, listeners.clone()).await,
            }
        }
    }

    async fn notify_request_dispatched(
        request: &oombak_sim::Request,
        listeners: Arc<RwLock<Listeners>>,
    ) {
        let response = Self::request_dispatched_notification(request);
        for listener in listeners.read().unwrap().iter() {
            listener
                .write()
                .unwrap()
                .on_receive_reponse(&response)
                .await;
        }
    }

    fn request_dispatched_notification(request: &oombak_sim::Request) -> oombak_sim::Response {
        let message = format!("`{}` request dispatched", request.payload);
        let notification = oombak_sim::response::Payload::generic_notification(message);
        oombak_sim::Response { id: request.id, payload: notification }
    }

    async fn notify(response: oombak_sim::response::Response, listeners: Arc<RwLock<Listeners>>) {
        for listener in listeners.read().unwrap().iter() {
            listener
                .write()
                .unwrap()
                .on_receive_reponse(&response)
                .await;
        }
    }
}

impl Thread for SimulatorRequestDispatcher {
    fn terminate(&mut self) -> ThreadResult {
        if let Some(handle) = self.handle.take() {
            self.channel
                .blocking_send(oombak_sim::request::Request::terminate())
                .unwrap();
            if let Err(e) = handle.join() {
                return Err(ThreadError::Panic(any_to_string(&e)));
            }
        }
        Ok(())
    }
}
