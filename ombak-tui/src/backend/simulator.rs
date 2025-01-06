use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

use bitvec::vec::BitVec;

use ombak::{dut::Dut, error::OmbakResult};

pub struct Simulator {
    request_tx: Sender<Request>,
    listeners: Arc<RwLock<Listeners>>,
}

type Listeners = Vec<Arc<RwLock<dyn Listener>>>;

pub trait Listener: Send + Sync {
    fn on_receive_reponse(&mut self, response: &Response);
}

pub enum Request {
    Run(u64),
    Load(String),
    GetSimulationResult,
    Terminate,
}

pub enum Response {
    RunResult(Result<(), String>),
    LoadResult(Result<(), String>),
    SimulationResult(Result<SimulationResult, String>),
}

impl Simulator {
    pub fn new() -> OmbakResult<Simulator> {
        let listeners = Arc::new(RwLock::new(vec![]));
        let (request_tx, request_rx) = mpsc::channel();
        Self::spawn_request_server(Arc::clone(&listeners), request_rx);
        Ok(Simulator {
            request_tx,
            listeners,
        })
    }

    pub fn register_listener(&mut self, listener: Arc<RwLock<dyn Listener>>) {
        self.listeners.write().unwrap().push(listener);
    }

    pub fn get_request_channel(&self) -> Sender<Request> {
        self.request_tx.clone()
    }

    fn spawn_request_server(listeners: Arc<RwLock<Listeners>>, request_rx: Receiver<Request>) {
        let mut server = RequestServer {
            dut: None,
            listeners,
        };
        let _ = thread::spawn(move || -> Result<(), String> {
            loop {
                match request_rx.recv().map_err(|e| e.to_string())? {
                    Request::Run(duration) => server.serve_run(duration),
                    Request::Load(lib_path) => server.serve_load(&lib_path),
                    Request::GetSimulationResult => server.serve_simulation_result(),
                    Request::Terminate => break (Ok(())),
                }
            }
        });
    }
}

struct RequestServer {
    dut: Option<Dut>,
    listeners: Arc<RwLock<Listeners>>,
}

impl RequestServer {
    fn serve_run(&self, _duration: u64) {
        let message = if let Some(_dut) = &self.dut {
            Response::RunResult(Ok(()))
        } else {
            Response::RunResult(Err("DUT not loaded".to_string()))
        };
        self.notify_listeners(message);
    }

    fn serve_load(&mut self, lib_path: &str) {
        let message = match Dut::new(lib_path) {
            Ok(dut) => {
                self.dut = Some(dut);
                Response::LoadResult(Ok(()))
            }
            Err(e) => Response::LoadResult(Err(e.to_string())),
        };
        self.notify_listeners(message);
    }

    fn serve_simulation_result(&mut self) {}

    fn notify_listeners(&self, message: Response) {
        for listener in self.listeners.read().unwrap().iter() {
            listener.write().unwrap().on_receive_reponse(&message);
        }
    }
}

#[derive(Clone, Default)]
pub struct SimulationResult {
    pub waves: Vec<Wave>,
    pub time_step_ps: usize,
}

#[derive(Clone)]
pub struct Wave {
    pub signal_name: String,
    pub width: usize,
    pub values: Vec<BitVec>,
}
