use std::path::{Path, PathBuf};

use async_trait::async_trait;
use bitvec::vec::BitVec;

use oombak_gen::TempGenDir;
use oombak_rs::{dut::Dut, probe::Probe};
use tokio::sync::{mpsc::Sender, RwLock};
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};
use tokio::task::spawn_blocking;

use crate::{
    error::{OombakSimError, OombakSimResult},
    request::{self},
    response::{self, LoadedDut, SimulationResult, Wave},
    Message, Request, Simulator,
};

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
    async fn serve(&self, request: Request) {
        let payload = match request.payload {
            request::Payload::Run(duration) => self.serve_run(duration).await,
            request::Payload::SetSignal(_, _) => todo!(),
            request::Payload::Load(path) => self.serve_load(&path).await,
            request::Payload::ModifyProbedPoints(_) => todo!(),
            request::Payload::GetSimulationResult => self.serve_simulation_result().await,
            request::Payload::Terminate => return self.drop_channel().await,
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
    async fn drop_channel(&self) {
        let mut current_channel = self.channel.write().await;
        *current_channel = None;
    }

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

    async fn serve_load(&self, sv_path: &Path) -> response::Payload {
        match self.load_dut(sv_path).await {
            Ok(loaded_dut) => response::Payload::from(loaded_dut),
            Err(e) => response::Payload::Error(Box::new(e)),
        }
    }

    async fn load_dut(&self, path: &Path) -> OombakSimResult<LoadedDut> {
        {
            let mut dut_state = self.dut_state.write().await;
            dut_state.will_be_reloaded = true;
        }

        let path_buf = path.to_path_buf();
        let (new_dut, temp_gen_dir, new_probe) =
            spawn_blocking(move || Self::generate_new_dut(&path_buf))
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

    fn generate_new_dut(path: &Path) -> OombakSimResult<(LoadedDut, TempGenDir, Probe)> {
        let (temp_gen_dir, probe) = oombak_gen::build(path)?;
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

    // fn serve_modify_probe_points(
    //     &self,
    //     probe_modifications: &ProbePointsModification,
    // ) -> response::Payload {
    // }

    // fn modify_probe_points(
    //     &mut self,
    //     probe_points_modification: &ProbePointsModification,
    // ) -> OombakSimResult<LoadedDut> {
    //     self.modify_probe(probe_points_modification)?;
    //     let temp_gen_dir = self.rebuild_sv_path()?;
    //     let lib_path = temp_gen_dir.lib_path();
    //     // TODO: Why is it necessary that we release resources here?
    //     self.release_resources();
    //     self.temp_gen_dir = Some(temp_gen_dir);
    //     self.dut = Some(Dut::new(lib_path.to_string_lossy().as_ref())?);
    //     self.reload_simulation_result()?;
    //     Ok(LoadedDut::from(
    //         self.probe.as_ref().ok_or(OombakSimError::DutNotLoaded)?,
    //     ))
    // }
    //
    //
    // fn rebuild_sv_path(&self) -> OombakSimResult<TempGenDir> {
    //     match (&self.sv_path, &self.probe) {
    //         (Some(sv_path), Some(probe)) => Ok(oombak_gen::build_with_probe(sv_path, probe)?),
    //         _ => Err(OombakSimError::DutNotLoaded),
    //     }
    // }
    //
    // fn set_signal(&self, signal_name: &str, value: &BitVec<u32>) -> OombakSimResult<()> {
    //     Ok(self.dut()?.set(signal_name, value)?)
    // }
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

    fn query(&self) -> OombakSimResult<Vec<oombak_rs::dut::Signal>> {
        match &self.dut {
            Some(dut) => Ok(dut.query()?),
            None => Err(OombakSimError::DutNotLoaded),
        }
    }

    // fn modify_probe(
    //     &mut self,
    //     probe_modifications: &ProbePointsModification,
    // ) -> OombakSimResult<()> {
    //     let probe = self.probe.as_mut().ok_or(OombakSimError::DutNotLoaded)?;
    //     for path in probe_modifications.to_add.iter() {
    //         probe.add_signal_to_probe(path)?;
    //     }
    //     for path in probe_modifications.to_remove.iter() {
    //         probe.remove_signal_from_probe(path)?;
    //     }
    //     Ok(())
    // }

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

    fn release_resources(&mut self) {
        _ = self.dut.take();
        _ = self.temp_gen_dir.take();
    }
}
