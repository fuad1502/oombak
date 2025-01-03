use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use ombak::{dut::Dut, error::OmbakResult};

pub struct Simulator {
    request_tx: Sender<Request>,
    listeners: Arc<Mutex<Vec<Arc<Mutex<dyn Listener + Send>>>>>,
}

pub trait Listener {
    fn on_receive_message(&mut self, message: &Message);
}

pub enum Request {
    Run(u64),
    Load(String),
    Terminate,
}

pub enum Message {
    RunResult(Result<(), String>),
    LoadResult(Result<(), String>),
}

impl Simulator {
    pub fn new() -> OmbakResult<Simulator> {
        let listeners = Arc::new(Mutex::new(vec![]));
        let (request_tx, request_rx) = mpsc::channel();
        Self::spawn_request_server(Arc::clone(&listeners), request_rx);
        Ok(Simulator {
            request_tx,
            listeners,
        })
    }

    pub fn register_listener(&mut self, listener: Arc<Mutex<dyn Listener + Send>>) {
        self.listeners.lock().unwrap().push(listener);
    }

    pub fn get_request_channel(&self) -> Sender<Request> {
        self.request_tx.clone()
    }

    fn spawn_request_server(
        listeners: Arc<Mutex<Vec<Arc<Mutex<dyn Listener + Send>>>>>,
        request_rx: Receiver<Request>,
    ) {
        let mut server = RequestServer {
            dut: None,
            listeners,
        };
        let _ = thread::spawn(move || -> Result<(), String> {
            loop {
                match request_rx.recv().map_err(|e| e.to_string())? {
                    Request::Run(duration) => server.serve_run(duration),
                    Request::Load(lib_path) => server.serve_load(&lib_path),
                    Request::Terminate => break (Ok(())),
                }
            }
        });
    }
}

struct RequestServer {
    dut: Option<Dut>,
    listeners: Arc<Mutex<Vec<Arc<Mutex<dyn Listener + Send>>>>>,
}

impl RequestServer {
    fn serve_run(&self, _duration: u64) {
        let message = if let Some(_dut) = &self.dut {
            Message::RunResult(Ok(()))
        } else {
            Message::RunResult(Err("DUT not loaded".to_string()))
        };
        self.notify_listeners(message);
    }

    fn serve_load(&mut self, lib_path: &str) {
        let message = match Dut::new(lib_path) {
            Ok(dut) => {
                self.dut = Some(dut);
                Message::LoadResult(Ok(()))
            }
            Err(e) => Message::LoadResult(Err(e.to_string())),
        };
        self.notify_listeners(message);
    }

    fn notify_listeners(&self, message: Message) {
        for listener in self.listeners.lock().unwrap().iter() {
            listener.lock().unwrap().on_receive_message(&message);
        }
    }
}
