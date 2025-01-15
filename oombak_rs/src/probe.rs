use std::sync::{Arc, RwLock};

use thiserror::Error;

use crate::{
    error::OombakResult,
    parser::{self, InstanceNode},
};

pub struct Probe {
    instance_node: Arc<RwLock<InstanceNode>>,
    probed_signals: Vec<parser::Signal>,
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("signal '{}' not available", _0)]
    UnknownSignal(String),
}

impl Probe {
    pub fn try_from(source_paths: &[&str], top_module_name: &str) -> OombakResult<Self> {
        let instance_node = parser::parse(source_paths, top_module_name)?;
        Ok(Self {
            instance_node,
            probed_signals: vec![],
        })
    }

    pub fn get_probed_signals(&self) -> &Vec<parser::Signal> {
        &self.probed_signals
    }

    pub fn add_signal_to_probe(&mut self, name: &str) -> OombakResult<()> {
        if let Some(signal) = self.instance_node.read().unwrap().get_signal(name) {
            self.probed_signals.push(signal);
            Ok(())
        } else {
            Err(ProbeError::UnknownSignal(name.to_string()).into())
        }
    }

    pub fn top_level_module_name(&self) -> String {
        let root = &self.instance_node.read().unwrap();
        root.module_name.clone()
    }
}
