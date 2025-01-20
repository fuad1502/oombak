use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

use bitvec::vec::BitVec;

use oombak_rs::{dut::Dut, error::OombakResult, probe::Probe};

use crate::error::{OombakSimError, OombakSimResult};

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
    Load(PathBuf),
    ModifyProbedPoints(ProbePointsModification),
    GetSimulationResult,
    Terminate,
}

pub enum Response<'a> {
    RunResult(Result<u64, String>),
    SetSignalResult(Result<(), String>),
    LoadResult(Result<LoadedDut, String>),
    ModifyProbedPointsResult(Result<LoadedDut, String>),
    SimulationResult(Result<&'a SimulationResult, String>),
}

pub use oombak_rs::parser::{InstanceNode, Signal, SignalType};

pub struct ProbePointsModification {
    pub to_add: Vec<String>,
    pub to_remove: Vec<String>,
}

pub struct LoadedDut {
    pub root_node: InstanceNode,
    pub probed_points: Vec<String>,
}

impl Simulator {
    pub fn new() -> OombakResult<Simulator> {
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
                    Request::Load(sv_path) => server.serve_load(&sv_path),
                    Request::ModifyProbedPoints(probe_points_modification) => {
                        server.serve_modify_probe_points(&probe_points_modification)
                    }
                    Request::GetSimulationResult => server.serve_simulation_result(),
                    Request::Terminate => break (Ok(())),
                }
            }
        });
    }
}

struct RequestServer {
    dut: Option<Dut>,
    probe: Option<Probe>,
    sv_path: Option<PathBuf>,
    listeners: Arc<RwLock<Listeners>>,
    simulation_time: u64,
    simulation_result: SimulationResult,
}

impl RequestServer {
    fn new(listeners: Arc<RwLock<Listeners>>) -> Self {
        Self {
            dut: None,
            sv_path: None,
            probe: None,
            listeners,
            simulation_time: 0,
            simulation_result: SimulationResult::default(),
        }
    }

    fn serve_run(&mut self, duration: u64) {
        let response = match self.run(duration) {
            Ok(duration) => Response::RunResult(Ok(duration)),
            Err(e) => Response::RunResult(Err(e.to_string())),
        };
        self.notify_listeners(response);
    }

    fn serve_set_signal(&self, signal_name: &str, value: &BitVec<u32>) {
        let response = match self.set_signal(signal_name, value) {
            Ok(_) => Response::SetSignalResult(Ok(())),
            Err(e) => Response::SetSignalResult(Err(e.to_string())),
        };
        self.notify_listeners(response);
    }

    fn serve_load(&mut self, sv_path: &Path) {
        let response = match self.load_dut(sv_path) {
            Ok(loaded_dut) => Response::LoadResult(Ok(loaded_dut)),
            Err(e) => Response::LoadResult(Err(e.to_string())),
        };
        self.notify_listeners(response);
    }

    fn serve_modify_probe_points(&mut self, probe_points_modification: &ProbePointsModification) {
        let response = match self.modify_probe_points(probe_points_modification) {
            Ok(loaded_dut) => Response::ModifyProbedPointsResult(Ok(loaded_dut)),
            Err(e) => Response::ModifyProbedPointsResult(Err(e.to_string())),
        };
        self.notify_listeners(response);
    }

    fn serve_simulation_result(&self) {
        let response = Response::SimulationResult(Ok(&self.simulation_result));
        self.notify_listeners(response);
    }

    fn load_dut(&mut self, sv_path: &Path) -> OombakSimResult<LoadedDut> {
        let (lib_path, probe) = oombak_gen::build(sv_path)?;
        let loaded_dut = LoadedDut::from(&probe);
        self.dut = Some(Dut::new(lib_path.to_string_lossy().as_ref())?);
        self.sv_path = Some(sv_path.to_path_buf());
        self.probe = Some(probe);
        self.reload_simulation_result()?;
        Ok(loaded_dut)
    }

    fn modify_probe_points(
        &mut self,
        probe_points_modification: &ProbePointsModification,
    ) -> OombakSimResult<LoadedDut> {
        self.modify_probe(probe_points_modification)?;
        let lib_path = self.rebuild_sv_path()?;
        self.dut = Some(Dut::new(lib_path.to_string_lossy().as_ref())?);
        self.reload_simulation_result()?;
        Ok(LoadedDut::from(
            self.probe.as_ref().ok_or(OombakSimError::DutNotLoaded)?,
        ))
    }

    fn modify_probe(
        &mut self,
        probe_points_modification: &ProbePointsModification,
    ) -> OombakSimResult<()> {
        let probe = self.probe.as_mut().ok_or(OombakSimError::DutNotLoaded)?;
        for path in probe_points_modification.to_add.iter() {
            probe.add_signal_to_probe(path)?;
        }
        Ok(())
    }

    fn rebuild_sv_path(&self) -> OombakSimResult<PathBuf> {
        match (&self.sv_path, &self.probe) {
            (Some(sv_path), Some(probe)) => Ok(oombak_gen::build_with_probe(sv_path, probe)?),
            _ => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn reload_simulation_result(&mut self) -> OombakSimResult<()> {
        self.simulation_result = SimulationResult::default();
        self.load_signal_names_to_simulation_result()?;
        Ok(())
    }

    fn load_signal_names_to_simulation_result(&mut self) -> OombakSimResult<()> {
        let waves: Vec<Wave> = self.dut()?.query()?.into_iter().map(Wave::from).collect();
        self.simulation_result.waves = waves;
        Ok(())
    }

    fn run(&mut self, duration: u64) -> OombakSimResult<u64> {
        let target_time = self.simulation_time + duration;
        while self.simulation_time != target_time {
            let curr_time = self.dut()?.run(duration)?;
            self.append_new_values_to_simulation_result_until(curr_time)?;
            self.simulation_time = curr_time;
        }
        Ok(self.simulation_time)
    }

    fn dut(&self) -> OombakSimResult<&Dut> {
        match &self.dut {
            Some(dut) => Ok(dut),
            None => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn append_new_values_to_simulation_result_until(
        &mut self,
        end_time: u64,
    ) -> OombakSimResult<()> {
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

    fn query_new_values(&self) -> OombakSimResult<Vec<BitVec<u32>>> {
        let mut new_values = vec![];
        for signal_name in self.simulation_result.waves.iter().map(|w| &w.signal_name) {
            let new_value = self.dut()?.get(signal_name)?;
            new_values.push(new_value);
        }
        Ok(new_values)
    }

    fn set_signal(&self, signal_name: &str, value: &BitVec<u32>) -> OombakSimResult<()> {
        Ok(self.dut()?.set(signal_name, value)?)
    }

    fn notify_listeners(&self, message: Response) {
        for listener in self.listeners.read().unwrap().iter() {
            listener.write().unwrap().on_receive_reponse(&message);
        }
    }
}

impl From<&Probe> for LoadedDut {
    fn from(probe: &Probe) -> Self {
        let probed_points = probe
            .get_probed_points()
            .iter()
            .map(|p| p.path().to_string())
            .collect();
        let root_node = probe.root_node().clone();
        LoadedDut {
            probed_points,
            root_node,
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

impl From<oombak_rs::dut::Signal> for Wave {
    fn from(signal: oombak_rs::dut::Signal) -> Self {
        Wave {
            signal_name: signal.name,
            width: signal.width as usize,
            values: vec![],
        }
    }
}
