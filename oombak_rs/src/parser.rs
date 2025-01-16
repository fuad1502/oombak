mod oombak_parser_sys;

use std::{
    ffi::{c_char, CStr, CString},
    sync::{Arc, RwLock, Weak},
};
use thiserror::Error;

use crate::error::{OombakError, OombakResult};

pub fn parse(
    source_paths: &[&str],
    top_module_name: &str,
) -> OombakResult<Arc<RwLock<InstanceNode>>> {
    let source_paths = CString::new(source_paths.join(":")).unwrap();
    let top_module_name = CString::new(top_module_name).unwrap();
    let instance_sys = unsafe {
        oombak_parser_sys::oombak_parser_parse(source_paths.as_ptr(), top_module_name.as_ptr())
    };
    InstanceNode::try_from(instance_sys)
}

#[derive(Default, Debug)]
pub struct InstanceNode {
    pub name: String,
    pub module_name: String,
    pub parent_node: Option<Weak<RwLock<InstanceNode>>>,
    pub children: Vec<Arc<RwLock<InstanceNode>>>,
    pub signals: Vec<Signal>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
    pub name: String,
    pub signal_type: SignalType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SignalType {
    UnpackedArrPort(Direction, usize),
    UnpackedArrNetVar(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    In,
    Out,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("null dereference")]
    NullDereference,
}

impl From<Error> for OombakError {
    fn from(value: Error) -> Self {
        OombakError::Parser(value)
    }
}

impl InstanceNode {
    fn try_from(
        ptr: *const oombak_parser_sys::Instance,
    ) -> OombakResult<Arc<RwLock<InstanceNode>>> {
        let instance = unsafe { deref_instance_ptr(&ptr)? };
        let name = string_from_ptr(instance.name)?;
        let module_name = string_from_ptr(instance.module_name)?;
        let signals = signals_ptr_to_vec(instance.signals, instance.signals_len as usize)?;
        let children = child_instances_ptr_to_vec(
            instance.child_instances,
            instance.child_instances_len as usize,
        )?;
        let node = InstanceNode {
            name,
            module_name,
            signals,
            children,
            parent_node: None,
        };
        Ok(Arc::new(RwLock::new(node)))
    }

    pub fn get_signal(&self, name: &str) -> Option<Signal> {
        if let Some((head, tail)) = name.split_once('.') {
            if self.name != head {
                return None;
            }
            for signal in self.signals.iter() {
                if signal.name == tail {
                    return Some(signal.clone());
                }
            }
            for child in self.children.iter() {
                let sig = child.read().unwrap().get_signal(tail);
                if sig.is_some() {
                    return sig;
                }
            }
            return None;
        }
        None
    }

    pub fn get_ports(&self) -> impl Iterator<Item = &Signal> {
        self.signals.iter().filter(|s| s.is_port())
    }
}

impl TryFrom<&oombak_parser_sys::Signal> for Signal {
    type Error = OombakError;

    fn try_from(value: &oombak_parser_sys::Signal) -> Result<Self, Self::Error> {
        let name = string_from_ptr(value.name)?;
        let width = value.width as usize;
        let signal_type = match value.signal_type {
            oombak_parser_sys::SignalType::UnpackedArrPortIn => {
                SignalType::UnpackedArrPort(Direction::In, width)
            }
            oombak_parser_sys::SignalType::UnpackedArrPortOut => {
                SignalType::UnpackedArrPort(Direction::Out, width)
            }
            oombak_parser_sys::SignalType::UnpackedArrVarNet => {
                SignalType::UnpackedArrNetVar(width)
            }
        };
        Ok(Signal { name, signal_type })
    }
}

impl Signal {
    pub fn is_port(&self) -> bool {
        match &self.signal_type {
            SignalType::UnpackedArrPort(_, _) => true,
            SignalType::UnpackedArrNetVar(_) => false,
        }
    }

    pub fn is_input_port(&self) -> bool {
        matches!(
            &self.signal_type,
            SignalType::UnpackedArrPort(Direction::In, _)
        )
    }

    pub fn bit_width(&self) -> usize {
        match &self.signal_type {
            SignalType::UnpackedArrPort(_, bit_width) => *bit_width,
            SignalType::UnpackedArrNetVar(bit_width) => *bit_width,
        }
    }
}

fn string_from_ptr(ptr: *const c_char) -> OombakResult<String> {
    if ptr.is_null() {
        return Err(Error::NullDereference.into());
    }
    Ok(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string())
}

unsafe fn deref_instance_ptr(
    ptr: &*const oombak_parser_sys::Instance,
) -> OombakResult<oombak_parser_sys::Instance> {
    if ptr.is_null() {
        return Err(Error::NullDereference.into());
    }
    Ok(unsafe { **ptr })
}

fn signals_ptr_to_vec(
    signals: *const oombak_parser_sys::Signal,
    signals_len: usize,
) -> OombakResult<Vec<Signal>> {
    if signals.is_null() {
        return Err(Error::NullDereference.into());
    }
    Ok(unsafe { std::slice::from_raw_parts(signals, signals_len) }
        .iter()
        .map(|s| Signal::try_from(s).unwrap())
        .collect())
}

fn child_instances_ptr_to_vec(
    child_instances: *const *const oombak_parser_sys::Instance,
    child_instances_len: usize,
) -> OombakResult<Vec<Arc<RwLock<InstanceNode>>>> {
    if child_instances.is_null() {
        return Err(Error::NullDereference.into());
    }
    Ok(
        unsafe { std::slice::from_raw_parts(child_instances, child_instances_len) }
            .iter()
            .map(|c| InstanceNode::try_from(*c).unwrap())
            .collect(),
    )
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use crate::parser::Direction;

    use super::{oombak_parser_sys::Instance, parse, InstanceNode, Signal, SignalType};

    #[test]
    fn test_get_signal() {
        let mut root = InstanceNode::default();
        root.name = "root".to_string();
        let root = Arc::new(RwLock::new(root));

        let mut child_0 = InstanceNode::default();
        child_0.name = "child_0".to_string();
        child_0.parent_node = Some(Arc::downgrade(&root));
        let child_0 = Arc::new(RwLock::new(child_0));

        let mut child_1 = InstanceNode::default();
        child_1.name = "child_1".to_string();
        child_1.parent_node = Some(Arc::downgrade(&root));
        let child_1 = Arc::new(RwLock::new(child_1));

        let mut child_2 = InstanceNode::default();
        child_2.name = "child_2".to_string();
        child_2.parent_node = Some(Arc::downgrade(&child_1));
        child_2.signals = vec![
            Signal {
                name: "sig_0".to_string(),
                signal_type: SignalType::UnpackedArrNetVar(1),
            },
            Signal {
                name: "sig_1".to_string(),
                signal_type: SignalType::UnpackedArrNetVar(1),
            },
        ];
        let child_2 = Arc::new(RwLock::new(child_2));

        root.write().unwrap().children = vec![Arc::clone(&child_0), Arc::clone(&child_1)];
        child_1.write().unwrap().children.push(Arc::clone(&child_2));

        assert!(root
            .read()
            .unwrap()
            .get_signal("root.child_1.child_2.sig_1")
            .is_some())
    }

    #[test]
    fn test_parse() {
        let source_paths = [
            "/home/fuad1502/code/oombak_parser/tests/fixtures/sv_sample_1/sample.sv",
            "/home/fuad1502/code/oombak_parser/tests/fixtures/sv_sample_1/adder.sv",
        ];
        let root = parse(&source_paths, "sample").unwrap();
        let root_read = root.read().unwrap();
        assert_eq!(root_read.name, "sample");
        assert_eq!(root_read.module_name, "sample");

        assert_eq!(root_read.signals.len(), 5);
        assert!(root_read.signals.contains(&Signal {
            name: "clk".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::In, 1)
        }));
        assert!(root_read.signals.contains(&Signal {
            name: "rst_n".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::In, 1)
        }));
        assert!(root_read.signals.contains(&Signal {
            name: "in".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::In, 6)
        }));
        assert!(root_read.signals.contains(&Signal {
            name: "out".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::Out, 6)
        }));
        assert!(root_read.signals.contains(&Signal {
            name: "c".to_string(),
            signal_type: SignalType::UnpackedArrNetVar(6)
        }));

        assert_eq!(root_read.children.len(), 1);
        let child = root_read.children[0].read().unwrap();

        assert_eq!(child.name, "adder_inst");
        assert_eq!(child.module_name, "adder");
        assert_eq!(child.children.len(), 0);
        assert_eq!(child.signals.len(), 4);
        assert!(child.signals.contains(&Signal {
            name: "a".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::In, 6)
        }));
        assert!(child.signals.contains(&Signal {
            name: "b".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::In, 6)
        }));
        assert!(child.signals.contains(&Signal {
            name: "c".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::Out, 6)
        }));
        assert!(child.signals.contains(&Signal {
            name: "d".to_string(),
            signal_type: SignalType::UnpackedArrNetVar(1)
        }));
    }

    #[test]
    fn test_null() {
        let ptr = 0 as *const Instance;
        let e = InstanceNode::try_from(ptr).unwrap_err();
        assert_eq!(&e.to_string(), "oombak_rs: parse: null dereference");
    }
}
