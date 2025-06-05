mod error;

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use bitvec::vec::BitVec;

use oombak_gen::TempGenDir;
use oombak_rs::{dut::Dut, probe::Probe};
use tokio::sync::{mpsc::Sender, RwLock};
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};
use tokio::task::spawn_blocking;

use oombak_sim::request::ProbePointsModification;
use oombak_sim::{
    request::{self},
    response::{self, LoadedDut, SimulationResult, Wave},
    Message, Request, Simulator,
};

use crate::error::{OombakSimError, OombakSimResult};

#[derive(Default)]
pub struct LocalSimulator {
    channel: RwLock<Option<Sender<Message>>>,
    simulation_result: RwLock<SimulationResult>,
    dut_state: RwLock<DutState>,
}

#[derive(Default)]
struct DutState {
    dut: Option<Dut>,
    probe: Option<Probe>,
    path: Option<PathBuf>,
    temp_gen_dir: Option<TempGenDir>,
    will_be_reloaded: bool,
}

#[async_trait]
impl Simulator for LocalSimulator {
    async fn serve(&self, request: &Request) {
        let payload = match &request.payload {
            request::Payload::Run(duration) => self.serve_run(*duration).await,
            request::Payload::SetSignal(signal_name, value) => {
                self.serve_set_signal(signal_name, value).await
            }
            request::Payload::Load(path) => self.serve_load(path, request.id).await,
            request::Payload::ModifyProbedPoints(probe_modifications) => {
                self.serve_modify_probe_points(probe_modifications, request.id)
                    .await
            }
            request::Payload::GetSimulationResult => self.serve_simulation_result().await,
            request::Payload::Terminate => return,
        };

        let channel = self.channel.read().await;
        if let Some(channel) = &*channel {
            channel
                .send(Message::Response(response::Response {
                    id: request.id,
                    payload,
                }))
                .await
                .unwrap();
        }
    }

    async fn set_channel(&self, channel: Sender<Message>) {
        let mut current_channel = self.channel.write().await;
        *current_channel = Some(channel);
    }
}

impl LocalSimulator {
    async fn serve_run(&self, duration: usize) -> response::Payload {
        let mut simulation_state = self.simulation_result.write().await;
        let dut_state = self.dut_state.read().await;
        match self.run(duration, &mut simulation_state, &dut_state) {
            Ok(current_time) => response::Payload::current_time(current_time),
            Err(e) => response::Payload::Error(Box::new(e)),
        }
    }

    fn run(
        &self,
        duration: usize,
        simulation_result: &mut RwLockWriteGuard<'_, SimulationResult>,
        dut_state: &RwLockReadGuard<'_, DutState>,
    ) -> OombakSimResult<usize> {
        let target_time = simulation_result.current_time + duration;
        while simulation_result.current_time != target_time {
            let curr_time = dut_state.run(duration)?;
            Self::append_new_values_to_simulation_result_until(
                curr_time,
                simulation_result,
                dut_state,
            )?;
        }
        Ok(simulation_result.current_time)
    }

    fn append_new_values_to_simulation_result_until(
        end_time: usize,
        simulation_result: &mut RwLockWriteGuard<'_, SimulationResult>,
        dut_state: &RwLockReadGuard<'_, DutState>,
    ) -> OombakSimResult<()> {
        let new_values = Self::query_new_values(simulation_result, dut_state)?;
        let current_time = simulation_result.current_time;
        let duration = end_time - current_time;
        for (wave, new_value) in simulation_result
            .waves
            .iter_mut()
            .zip(new_values.into_iter())
        {
            if let Some((value, _start, count)) = wave.values.last_mut() {
                if *value == new_value {
                    *count += duration;
                } else {
                    wave.values.push((new_value, current_time, duration));
                }
            } else {
                wave.values.push((new_value, current_time, duration));
            }
        }
        simulation_result.current_time = end_time;
        Ok(())
    }

    fn query_new_values(
        simulation_result: &mut RwLockWriteGuard<'_, SimulationResult>,
        dut_state: &RwLockReadGuard<'_, DutState>,
    ) -> OombakSimResult<Vec<BitVec<u32>>> {
        let mut new_values = vec![];
        for signal_name in simulation_result.waves.iter().map(|w| &w.signal_name) {
            let new_value = dut_state.get(signal_name)?;
            new_values.push(new_value);
        }
        Ok(new_values)
    }

    async fn serve_load(&self, sv_path: &Path, message_id: usize) -> response::Payload {
        match self.load_dut(sv_path, message_id).await {
            Ok(loaded_dut) => response::Payload::from(loaded_dut),
            Err(e) => response::Payload::Error(Box::new(e)),
        }
    }

    async fn load_dut(&self, path: &Path, message_id: usize) -> OombakSimResult<LoadedDut> {
        {
            let mut dut_state = self.dut_state.write().await;
            if dut_state.will_be_reloaded {
                return Err(OombakSimError::DutIsLoading);
            }
            dut_state.will_be_reloaded = true;
        }

        let path_buf = path.to_path_buf();
        let notification_channel = self.channel.read().await.clone();
        let (new_dut, temp_gen_dir, new_probe) = spawn_blocking(move || {
            Self::generate_new_dut(&path_buf, notification_channel, message_id)
        })
        .await
        .unwrap()?;

        {
            let mut dut_state = self.dut_state.write().await;
            dut_state.reload(path, temp_gen_dir, new_probe)?;
        }

        let dut_state = self.dut_state.read().await;
        let mut simulation_result = self.simulation_result.write().await;
        Self::reload_simulation_result(&mut simulation_result, &dut_state)?;

        Ok(new_dut)
    }

    fn generate_new_dut(
        path: &Path,
        notification_channel: Option<Sender<Message>>,
        message_id: usize,
    ) -> OombakSimResult<(LoadedDut, TempGenDir, Probe)> {
        let builder = oombak_gen::Builder::new(notification_channel, message_id);
        let (temp_gen_dir, probe) = builder.build(path)?;
        let loaded_dut = LoadedDut::from(&probe);
        Ok((loaded_dut, temp_gen_dir, probe))
    }

    fn reload_simulation_result(
        simulation_result: &mut RwLockWriteGuard<'_, SimulationResult>,
        dut_state: &RwLockReadGuard<'_, DutState>,
    ) -> OombakSimResult<()> {
        **simulation_result = SimulationResult::default();
        Self::load_signal_names_to_simulation_result(simulation_result, dut_state)?;
        Ok(())
    }

    fn load_signal_names_to_simulation_result(
        simulation_result: &mut RwLockWriteGuard<'_, SimulationResult>,
        dut_state: &RwLockReadGuard<'_, DutState>,
    ) -> OombakSimResult<()> {
        let waves: Vec<Wave> = dut_state.query()?.into_iter().map(Wave::from).collect();
        simulation_result.waves = waves;
        Ok(())
    }

    async fn serve_simulation_result(&self) -> response::Payload {
        let simulation_result = self.simulation_result.read().await;
        response::Payload::from(simulation_result.clone())
    }

    async fn serve_set_signal(&self, signal_name: &str, value: &BitVec<u32>) -> response::Payload {
        let dut_state = self.dut_state.read().await;
        match dut_state.set(signal_name, value) {
            Ok(()) => response::Payload::empty(),
            Err(e) => response::Payload::Error(Box::new(e)),
        }
    }

    async fn serve_modify_probe_points(
        &self,
        probe_modifications: &ProbePointsModification,
        message_id: usize,
    ) -> response::Payload {
        match self
            .modify_probe_points(probe_modifications, message_id)
            .await
        {
            Ok(dut) => response::Payload::from(dut),
            Err(e) => response::Payload::Error(Box::new(e)),
        }
    }

    async fn modify_probe_points(
        &self,
        probe_modifications: &ProbePointsModification,
        message_id: usize,
    ) -> OombakSimResult<LoadedDut> {
        {
            let mut dut_state = self.dut_state.write().await;
            if dut_state.will_be_reloaded {
                return Err(OombakSimError::DutIsLoading);
            }
            dut_state.will_be_reloaded = true;
        }

        let new_probe = {
            let dut_state = self.dut_state.read().await;
            let probe = dut_state.probe()?;
            Self::get_modified_probe(probe, probe_modifications)?
        };
        let new_probe_clone = new_probe.clone();

        let path = {
            let dut_state = self.dut_state.read().await;
            let path = dut_state.path()?;
            PathBuf::from(path)
        };

        let notification_channel = self.channel.read().await.clone();

        let (new_dut, temp_gen_dir) = spawn_blocking(move || {
            Self::regenerate_dut(&path, &new_probe, notification_channel, message_id)
        })
        .await
        .unwrap()?;

        {
            let mut dut_state = self.dut_state.write().await;
            dut_state.reload_path_unchanged(temp_gen_dir, new_probe_clone)?;
        }

        let dut_state = self.dut_state.read().await;
        let mut simulation_result = self.simulation_result.write().await;
        Self::reload_simulation_result(&mut simulation_result, &dut_state)?;

        Ok(new_dut)
    }

    fn get_modified_probe(
        probe: &Probe,
        probe_modifications: &ProbePointsModification,
    ) -> OombakSimResult<Probe> {
        let mut new_probe = probe.clone();
        for path in probe_modifications.to_add.iter() {
            new_probe.add_signal_to_probe(path)?;
        }
        for path in probe_modifications.to_remove.iter() {
            new_probe.remove_signal_from_probe(path)?;
        }
        Ok(new_probe)
    }

    fn regenerate_dut(
        path: &Path,
        probe: &Probe,
        notification_channel: Option<Sender<Message>>,
        message_id: usize,
    ) -> OombakSimResult<(LoadedDut, TempGenDir)> {
        let builder = oombak_gen::Builder::new(notification_channel, message_id);
        let temp_gen_dir = builder.build_with_probe(path, probe)?;
        let loaded_dut = LoadedDut::from(probe);
        Ok((loaded_dut, temp_gen_dir))
    }
}

impl DutState {
    fn run(&self, duration: usize) -> OombakSimResult<usize> {
        match &self.dut {
            Some(dut) => Ok(dut.run(duration as u64)? as usize),
            None => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn get(&self, signal_name: &str) -> OombakSimResult<BitVec<u32>> {
        match &self.dut {
            Some(dut) => Ok(dut.get(signal_name)?),
            None => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn set(&self, signal_name: &str, value: &BitVec<u32>) -> OombakSimResult<()> {
        match &self.dut {
            Some(dut) => Ok(dut.set(signal_name, value)?),
            None => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn query(&self) -> OombakSimResult<Vec<oombak_rs::dut::Signal>> {
        match &self.dut {
            Some(dut) => Ok(dut.query()?),
            None => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn probe(&self) -> OombakSimResult<&Probe> {
        match &self.probe {
            Some(probe) => Ok(probe),
            None => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn path(&self) -> OombakSimResult<&Path> {
        match &self.path {
            Some(path) => Ok(path),
            None => Err(OombakSimError::DutNotLoaded),
        }
    }

    fn reload(
        &mut self,
        sv_path: &Path,
        temp_gen_dir: TempGenDir,
        probe: Probe,
    ) -> OombakSimResult<()> {
        self.release_resources();
        let lib_path = temp_gen_dir.lib_path();
        self.temp_gen_dir = Some(temp_gen_dir);
        self.dut = Some(Dut::new(lib_path.to_string_lossy().as_ref())?);
        self.path = Some(sv_path.to_path_buf());
        self.probe = Some(probe);
        self.will_be_reloaded = false;
        Ok(())
    }

    fn reload_path_unchanged(
        &mut self,
        temp_gen_dir: TempGenDir,
        probe: Probe,
    ) -> OombakSimResult<()> {
        self.release_resources();
        let lib_path = temp_gen_dir.lib_path();
        self.temp_gen_dir = Some(temp_gen_dir);
        self.dut = Some(Dut::new(lib_path.to_string_lossy().as_ref())?);
        self.probe = Some(probe);
        self.will_be_reloaded = false;
        Ok(())
    }

    fn release_resources(&mut self) {
        _ = self.dut.take();
        _ = self.temp_gen_dir.take();
    }
}
