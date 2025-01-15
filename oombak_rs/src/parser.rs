mod oombak_parser_sys;

use std::{
    ffi::{c_char, CStr, CString},
    sync::{Arc, RwLock, Weak},
};

pub fn parse(source_paths: &[&str], top_module_name: &str) -> Result<InstanceNode, String> {
    let source_paths = CString::new(source_paths.join(":")).unwrap();
    let top_module_name = CString::new(top_module_name).unwrap();
    let instance_sys = unsafe {
        oombak_parser_sys::oombak_parser_parse(source_paths.as_ptr(), top_module_name.as_ptr())
    };
    InstanceNode::try_from(instance_sys)
}

#[derive(Default)]
pub struct InstanceNode {
    pub name: String,
    pub module_name: String,
    pub parent_node: Weak<RwLock<InstanceNode>>,
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

impl InstanceNode {
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
}

impl TryFrom<*const oombak_parser_sys::Instance> for InstanceNode {
    type Error = String;

    fn try_from(ptr: *const oombak_parser_sys::Instance) -> Result<Self, Self::Error> {
        let instance = unsafe { deref_instance_ptr(&ptr) };
        let name = string_from_ptr(instance.name);
        let module_name = string_from_ptr(instance.module_name);
        let signals = signals_ptr_to_vec(instance.signals, instance.signals_len as usize);
        let children = child_instances_ptr_to_vec(
            instance.child_instances,
            instance.child_instances_len as usize,
        );
        Ok(InstanceNode {
            name,
            module_name,
            signals,
            children,
            parent_node: Weak::default(),
        })
    }
}

impl TryFrom<&oombak_parser_sys::Signal> for Signal {
    type Error = String;

    fn try_from(value: &oombak_parser_sys::Signal) -> Result<Self, Self::Error> {
        let name = string_from_ptr(value.name);
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

fn string_from_ptr(ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_string()
}

unsafe fn deref_instance_ptr(
    ptr: &*const oombak_parser_sys::Instance,
) -> oombak_parser_sys::Instance {
    unsafe { **ptr }
}

fn signals_ptr_to_vec(
    signals: *const oombak_parser_sys::Signal,
    signals_len: usize,
) -> Vec<Signal> {
    unsafe { std::slice::from_raw_parts(signals, signals_len) }
        .iter()
        .map(|s| Signal::try_from(s).unwrap())
        .collect()
}

fn child_instances_ptr_to_vec(
    child_instances: *const *const oombak_parser_sys::Instance,
    child_instances_len: usize,
) -> Vec<Arc<RwLock<InstanceNode>>> {
    unsafe { std::slice::from_raw_parts(child_instances, child_instances_len) }
        .iter()
        .map(|c| Arc::new(RwLock::new(InstanceNode::try_from(*c).unwrap())))
        .collect()
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use crate::parser::Direction;

    use super::{parse, InstanceNode, Signal, SignalType};

    #[test]
    fn test_get_signal() {
        let mut root = InstanceNode::default();
        root.name = "root".to_string();
        let root = Arc::new(RwLock::new(root));

        let mut child_0 = InstanceNode::default();
        child_0.name = "child_0".to_string();
        child_0.parent_node = Arc::downgrade(&root);
        let child_0 = Arc::new(RwLock::new(child_0));

        let mut child_1 = InstanceNode::default();
        child_1.name = "child_1".to_string();
        child_1.parent_node = Arc::downgrade(&root);
        let child_1 = Arc::new(RwLock::new(child_1));

        let mut child_2 = InstanceNode::default();
        child_2.name = "child_2".to_string();
        child_2.parent_node = Arc::downgrade(&child_1);
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
        assert_eq!(root.name, "sample");
        assert_eq!(root.module_name, "sample");

        assert_eq!(root.signals.len(), 5);
        assert!(root.signals.contains(&Signal {
            name: "clk".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::In, 1)
        }));
        assert!(root.signals.contains(&Signal {
            name: "rst_n".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::In, 1)
        }));
        assert!(root.signals.contains(&Signal {
            name: "in".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::In, 6)
        }));
        assert!(root.signals.contains(&Signal {
            name: "out".to_string(),
            signal_type: SignalType::UnpackedArrPort(Direction::Out, 6)
        }));
        assert!(root.signals.contains(&Signal {
            name: "c".to_string(),
            signal_type: SignalType::UnpackedArrNetVar(6)
        }));

        assert_eq!(root.children.len(), 1);
        let child = root.children[0].read().unwrap();

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
}
