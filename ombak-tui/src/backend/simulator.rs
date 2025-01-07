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
    SetSignal(String, BitVec<u32>),
    Load(String),
    GetSimulationResult,
    Terminate,
}

pub enum Response<'a> {
    RunResult(Result<u64, String>),
    SetSignalResult(Result<(), String>),
    LoadResult(Result<(), String>),
    SimulationResult(Result<&'a SimulationResult, String>),
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
        let mut server = RequestServer::new(listeners);
        let _ = thread::spawn(move || -> Result<(), String> {
            loop {
                match request_rx.recv().map_err(|e| e.to_string())? {
                    Request::Run(duration) => server.serve_run(duration),
                    Request::SetSignal(signal_name, value) => {
                        server.serve_set_signal(&signal_name, &value)
                    }
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
    simulation_time: u64,
    simulation_result: SimulationResult,
}

impl RequestServer {
    const DUT_NOT_LOADED: &str = "DUT not loaded";

    fn new(listeners: Arc<RwLock<Listeners>>) -> Self {
        Self {
            dut: None,
            listeners,
            simulation_time: 0,
            simulation_result: SimulationResult::default(),
        }
    }

    fn serve_run(&mut self, duration: u64) {
        let response = match self.run(duration) {
            Ok(duration) => Response::RunResult(Ok(duration)),
            Err(e) => Response::RunResult(Err(e)),
        };
        self.notify_listeners(response);
    }

    fn serve_set_signal(&self, signal_name: &str, value: &BitVec<u32>) {
        let response = match self.set_signal(signal_name, value) {
            Ok(_) => Response::SetSignalResult(Ok(())),
            Err(e) => Response::SetSignalResult(Err(e)),
        };
        self.notify_listeners(response);
    }

    fn serve_load(&mut self, lib_path: &str) {
        let response = match Dut::new(lib_path) {
            Ok(dut) => match self.load_new_dut(dut) {
                Ok(()) => Response::LoadResult(Ok(())),
                Err(e) => Response::LoadResult(Err(e.to_string())),
            },
            Err(e) => Response::LoadResult(Err(e.to_string())),
        };
        self.notify_listeners(response);
    }

    fn serve_simulation_result(&self) {
        let response = Response::SimulationResult(Ok(&self.simulation_result));
        self.notify_listeners(response);
    }

    fn load_new_dut(&mut self, dut: Dut) -> Result<(), String> {
        self.simulation_result = SimulationResult::default();
        self.dut = Some(dut);
        self.load_signal_names_to_simulation_result()?;
        Ok(())
    }

    fn load_signal_names_to_simulation_result(&mut self) -> Result<(), String> {
        let waves: Vec<Wave> = self.dut()?.query()?.into_iter().map(Wave::from).collect();
        self.simulation_result.waves = waves;
        Ok(())
    }

    fn run(&mut self, duration: u64) -> Result<u64, String> {
        let target_time = self.simulation_time + duration;
        while self.simulation_time != target_time {
            let curr_time = self.dut()?.run(duration)?;
            self.append_new_values_to_simulation_result_until(curr_time)?;
            self.simulation_time = curr_time;
        }
        Ok(self.simulation_time)
    }

    fn dut(&self) -> Result<&Dut, String> {
        match &self.dut {
            Some(dut) => Ok(dut),
            None => Err(Self::DUT_NOT_LOADED.to_string()),
        }
    }

    fn append_new_values_to_simulation_result_until(
        &mut self,
        end_time: u64,
    ) -> Result<(), String> {
        let new_values = self.query_new_values()?;
        for (wave, new_value) in self
            .simulation_result
            .waves
            .iter_mut()
            .zip(new_values.into_iter())
        {
            for _ in 0..(end_time - self.simulation_time) {
                wave.values.push(new_value.clone());
            }
        }
        Ok(())
    }

    fn query_new_values(&self) -> Result<Vec<BitVec<u32>>, String> {
        let mut new_values = vec![];
        for signal_name in self.simulation_result.waves.iter().map(|w| &w.signal_name) {
            let new_value = self.dut()?.get(signal_name)?;
            new_values.push(new_value);
        }
        Ok(new_values)
    }

    fn set_signal(&self, signal_name: &str, value: &BitVec<u32>) -> Result<(), String> {
        Ok(self.dut()?.set(signal_name, value)?)
    }

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
    pub values: Vec<BitVec<u32>>,
}

impl From<ombak::dut::Signal> for Wave {
    fn from(signal: ombak::dut::Signal) -> Self {
        Wave {
            signal_name: signal.name,
            width: signal.width as usize,
            values: vec![],
        }
    }
}
