use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, RwLock,
    },
    thread,
};

use bitvec::vec::BitVec;

use oombak_gen::TempGenDir;
use oombak_rs::{dut::Dut, error::OombakResult, probe::Probe};

use crate::{
    error::{OombakSimError, OombakSimResult},
    request::{self, ProbePointsModification},
    response::{self, LoadedDut, SimulationResult, Wave},
    Request, Response,
};

pub struct Simulator {
    request_tx: Sender<Request>,
    listeners: Arc<RwLock<Listeners>>,
}

type Listeners = Vec<Arc<RwLock<dyn Listener>>>;

pub trait Listener: Send + Sync {
    fn on_receive_reponse(&mut self, response: &Response);
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
        let _ = thread::spawn(move || -> Result<(), String> {
            let mut server = RequestServer::new(listeners);
            loop {
                let request = request_rx.recv().map_err(|e| e.to_string())?;
                server.serving_id = request.id;
                match request.payload {
                    request::Payload::Run(duration) => server.serve_run(duration),
                    request::Payload::SetSignal(signal_name, value) => {
                        server.serve_set_signal(&signal_name, &value)
                    }
                    request::Payload::Load(sv_path) => server.serve_load(&sv_path),
                    request::Payload::ModifyProbedPoints(probe_points_modification) => {
                        server.serve_modify_probe_points(&probe_points_modification)
                    }
                    request::Payload::GetSimulationResult => server.serve_simulation_result(),
                    request::Payload::Terminate => {
                        // TODO: Why is it necessary that we release resources here?
                        server.release_resources();
                        break (Ok(()));
                    }
                }
            }
        });
    }
}

struct RequestServer {
    serving_id: usize,
    dut: Option<Dut>,
    probe: Option<Probe>,
    sv_path: Option<PathBuf>,
    temp_gen_dir: Option<TempGenDir>,
    listeners: Arc<RwLock<Listeners>>,
    simulation_time: u64,
    simulation_result: SimulationResult,
}

impl RequestServer {
    fn new(listeners: Arc<RwLock<Listeners>>) -> Self {
        Self {
            serving_id: 0,
            dut: None,
            sv_path: None,
            temp_gen_dir: None,
            probe: None,
            listeners,
            simulation_time: 0,
            simulation_result: SimulationResult::default(),
        }
    }

    fn serve_run(&mut self, duration: u64) {
        let response = match self.run(duration) {
            Ok(current_time) => response::Payload::current_time(current_time),
            Err(e) => response::Payload::Error(Box::new(e)),
        };
        self.notify_listeners(response);
    }

    fn serve_set_signal(&self, signal_name: &str, value: &BitVec<u32>) {
        let payload = match self.set_signal(signal_name, value) {
            Ok(_) => response::Payload::empty(),
            Err(e) => response::Payload::Error(Box::new(e)),
        };
        self.notify_listeners(payload);
    }

    fn serve_load(&mut self, sv_path: &Path) {
        let payload = match self.load_dut(sv_path) {
            Ok(loaded_dut) => response::Payload::from(loaded_dut),
            Err(e) => response::Payload::Error(Box::new(e)),
        };
        self.notify_listeners(payload);
    }

    fn serve_modify_probe_points(&mut self, probe_points_modification: &ProbePointsModification) {
        let payload = match self.modify_probe_points(probe_points_modification) {
            Ok(loaded_dut) => response::Payload::from(loaded_dut),
            Err(e) => response::Payload::Error(Box::new(e)),
        };
        self.notify_listeners(payload);
    }

    fn serve_simulation_result(&self) {
        let result = response::Results::SimulationResult(&self.simulation_result);
        self.notify_listeners(response::Payload::Result(result));
    }

    fn release_resources(&mut self) {
        _ = self.dut.take();
        _ = self.temp_gen_dir.take();
    }

    fn load_dut(&mut self, sv_path: &Path) -> OombakSimResult<LoadedDut> {
        let (temp_gen_dir, probe) = oombak_gen::build(sv_path)?;
        let loaded_dut = LoadedDut::from(&probe);
        let lib_path = temp_gen_dir.lib_path();
        // TODO: Why is it necessary that we release resources here?
        self.release_resources();
        self.temp_gen_dir = Some(temp_gen_dir);
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
        let temp_gen_dir = self.rebuild_sv_path()?;
        let lib_path = temp_gen_dir.lib_path();
        // TODO: Why is it necessary that we release resources here?
        self.release_resources();
        self.temp_gen_dir = Some(temp_gen_dir);
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
        for path in probe_points_modification.to_remove.iter() {
            probe.remove_signal_from_probe(path)?;
        }
        Ok(())
    }

    fn rebuild_sv_path(&self) -> OombakSimResult<TempGenDir> {
        match (&self.sv_path, &self.probe) {
            (Some(sv_path), Some(probe)) => Ok(oombak_gen::build_with_probe(sv_path, probe)?),
            _ => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn reload_simulation_result(&mut self) -> OombakSimResult<()> {
        self.simulation_result = SimulationResult::default();
        self.simulation_time = 0;
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
        let duration = (end_time - self.simulation_time) as usize;
        self.simulation_result.total_time += duration;
        for (wave, new_value) in self
            .simulation_result
            .waves
            .iter_mut()
            .zip(new_values.into_iter())
        {
            if let Some((value, _start, count)) = wave.values.last_mut() {
                if *value == new_value {
                    *count += duration;
                } else {
                    wave.values
                        .push((new_value, self.simulation_time as usize, duration));
                }
            } else {
                wave.values
                    .push((new_value, self.simulation_time as usize, duration));
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

    fn notify_listeners(&self, payload: response::Payload) {
        let response = Response {
            id: self.serving_id,
            payload,
        };
        for listener in self.listeners.read().unwrap().iter() {
            listener.write().unwrap().on_receive_reponse(&response);
        }
    }
}
