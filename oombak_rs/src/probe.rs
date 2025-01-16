use std::sync::{Arc, RwLock};

use thiserror::Error;

use crate::{
    error::{OombakError, OombakResult},
    parser::{self, InstanceNode, Signal},
};

pub struct Probe {
    root_node: Arc<RwLock<InstanceNode>>,
    points: Vec<ProbePoint>,
}

pub struct ProbePoint {
    path: String,
    signal: parser::Signal,
    is_top_level_input: bool,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("signal '{}' not available", _0)]
    UnknownSignal(String),
}

impl From<Error> for OombakError {
    fn from(value: Error) -> Self {
        OombakError::Probe(value)
    }
}

impl Probe {
    pub fn try_from(source_paths: &[&str], top_module_name: &str) -> OombakResult<Self> {
        let instance_node = parser::parse(source_paths, top_module_name)?;
        let points = Self::create_top_level_points(&instance_node);
        Ok(Self {
            root_node: instance_node,
            points,
        })
    }

    pub fn get_probed_points(&self) -> &Vec<ProbePoint> {
        &self.points
    }

    pub fn get_top_level_ports(&self) -> Vec<parser::Signal> {
        self.root_node
            .read()
            .unwrap()
            .get_ports()
            .map(|s| s.to_owned())
            .collect()
    }

    pub fn get_settable_points(&self) -> impl Iterator<Item = &ProbePoint> {
        self.points.iter().filter(|p| p.is_top_level_input)
    }

    pub fn get_gettable_points(&self) -> impl Iterator<Item = &ProbePoint> {
        self.points.iter().filter(|_| true)
    }

    pub fn get_multibit_settable_points(&self) -> impl Iterator<Item = &ProbePoint> {
        self.points
            .iter()
            .filter(|p| p.is_top_level_input && p.bit_width() > 1)
    }

    pub fn get_multibit_gettable_points(&self) -> impl Iterator<Item = &ProbePoint> {
        self.points.iter().filter(|p| p.bit_width() > 1)
    }

    pub fn get_single_bit_settable_points(&self) -> impl Iterator<Item = &ProbePoint> {
        self.points
            .iter()
            .filter(|p| p.is_top_level_input && p.bit_width() == 1)
    }

    pub fn get_single_bit_gettable_points(&self) -> impl Iterator<Item = &ProbePoint> {
        self.points.iter().filter(|p| p.bit_width() == 1)
    }

    pub fn add_signal_to_probe(&mut self, path: &str) -> OombakResult<()> {
        if let Ok(Some(signal)) = self.root_node.read().unwrap().get_signal(path) {
            let probe_point = ProbePoint {
                path: path.to_string(),
                signal,
                is_top_level_input: false,
            };
            self.points.push(probe_point);
            Ok(())
        } else {
            Err(Error::UnknownSignal(path.to_string()).into())
        }
    }

    pub fn top_level_module_name(&self) -> String {
        let root = &self.root_node.read().unwrap();
        root.module_name.clone()
    }

    fn create_top_level_points(root_node: &Arc<RwLock<InstanceNode>>) -> Vec<ProbePoint> {
        root_node
            .read()
            .unwrap()
            .get_ports()
            .map(|s| {
                let path = s
                    .name
                    .split(".")
                    .last()
                    .expect("path must contain at least a single period")
                    .to_string();
                let signal = Signal {
                    name: path.clone(),
                    signal_type: s.signal_type.clone(),
                };
                let is_top_level_input = signal.is_input_port();
                ProbePoint {
                    path,
                    signal,
                    is_top_level_input,
                }
            })
            .collect()
    }
}

impl ProbePoint {
    pub fn get_dot_replaced_path(&self) -> String {
        self.path.replace(".", "_DOT_")
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn bit_width(&self) -> usize {
        self.signal.bit_width()
    }

    pub fn is_gettable(&self) -> bool {
        true
    }

    pub fn is_settable(&self) -> bool {
        self.is_top_level_input
    }
}
