use std::sync::{Arc, RwLock, Weak};

#[derive(Default)]
pub struct InstanceNode {
    pub name: String,
    pub module_name: String,
    pub parent_node: Weak<RwLock<InstanceNode>>,
    pub children: Vec<Arc<RwLock<InstanceNode>>>,
    pub signals: Vec<Signal>,
}

#[derive(Clone)]
pub struct Signal {
    pub name: String,
    pub signal_type: SignalType,
}

#[derive(Clone)]
pub enum SignalType {
    UnpackedArrPort(Direction, usize),
    UnpackedArrNetVar(usize),
}

#[derive(Clone)]
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

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};

    use super::{InstanceNode, Signal, SignalType};

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
}
