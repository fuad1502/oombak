mod oombak_parser_sys;

use std::{
    ffi::{c_char, CStr, CString},
    fmt::Display,
};

use crate::{probe, OombakResult};

pub fn parse(source_paths: &[String], top_level_module_name: &str) -> OombakResult<InstanceNode> {
    let source_paths = CString::new(source_paths.join(":"))?;
    let top_level_module_name = CString::new(top_level_module_name)?;
    let ctx = unsafe { oombak_parser_sys::oombak_parser_get_ctx() };
    let parse_res = unsafe {
        oombak_parser_sys::oombak_parser_parse_r(
            ctx,
            source_paths.as_ptr(),
            top_level_module_name.as_ptr(),
        )
    };
    check_compile_error(&parse_res, ctx)?;
    let result = InstanceNode::try_from(parse_res);
    unsafe { oombak_parser_sys::oombak_parser_free_ctx(ctx) };
    result
}

fn check_compile_error(
    parse_res: &oombak_parser_sys::Result,
    ctx: oombak_parser_sys::Context,
) -> OombakResult<()> {
    if parse_res.is_error == 1
        && matches!(
            unsafe { parse_res.instance_or_error.error },
            oombak_parser_sys::Error::CompileError
        )
    {
        let error_message = unsafe { oombak_parser_sys::oombak_parser_get_last_diagnostics_r(ctx) };
        let error_message =
            String::from_utf8_lossy((unsafe { CStr::from_ptr(error_message) }).to_bytes())
                .to_string();
        return Err(Error::FailedToCompile(error_message).into());
    }
    Ok(())
}

#[derive(Default, Debug, Clone)]
pub struct InstanceNode {
    pub name: String,
    pub module_name: String,
    pub children: Vec<InstanceNode>,
    pub signals: Vec<Signal>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
    pub name: String,
    pub signal_type: SignalType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SignalType {
    PackedArrPort(Direction, usize),
    PackedArrNetVar(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    In,
    Out,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("null dereference")]
    NullDereference,
    #[error("file not found")]
    FileNotFound,
    #[error("top-level module not found")]
    TopLevelModuleNotFound,
    #[error("failed to compile:\n{}", _0)]
    FailedToCompile(String),
    #[error("found unsupported symbol type")]
    UnsupportedSymbolType,
    #[error("found unsupported port direction")]
    UnsupportedPortDirection,
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Probe(probe::Error::Parser(value))
    }
}

impl InstanceNode {
    pub fn get_signal(&self, name: &str) -> OombakResult<Option<Signal>> {
        if let Some((head, tail)) = name.split_once('.') {
            if self.name != head {
                return Ok(None);
            }
            for signal in self.signals.iter() {
                if signal.name == tail {
                    return Ok(Some(signal.clone()));
                }
            }
            for child in self.children.iter() {
                let sig = child.get_signal(tail)?;
                if sig.is_some() {
                    return Ok(sig);
                }
            }
            return Ok(None);
        }
        Ok(None)
    }

    pub fn get_ports(&self) -> impl Iterator<Item = &Signal> {
        self.signals.iter().filter(|s| s.is_port())
    }
}

impl TryFrom<oombak_parser_sys::Result> for InstanceNode {
    type Error = crate::Error;
    fn try_from(value: oombak_parser_sys::Result) -> OombakResult<InstanceNode> {
        if value.is_error > 0 {
            return match unsafe { value.instance_or_error.error } {
                oombak_parser_sys::Error::FileNotFound => Err(Error::FileNotFound.into()),
                oombak_parser_sys::Error::TopLevelModuleNotFound => {
                    Err(Error::TopLevelModuleNotFound.into())
                }
                oombak_parser_sys::Error::CompileError => {
                    Err(Error::FailedToCompile(String::new()).into())
                }
                oombak_parser_sys::Error::UnsupportedSymbolType => {
                    Err(Error::UnsupportedSymbolType.into())
                }
                oombak_parser_sys::Error::UnsupportedPortDirection => {
                    Err(Error::UnsupportedPortDirection.into())
                }
                oombak_parser_sys::Error::None => unreachable!(),
            };
        }
        InstanceNode::try_from(unsafe { &value.instance_or_error.instance })
    }
}

impl TryFrom<&*const oombak_parser_sys::Instance> for InstanceNode {
    type Error = crate::Error;

    fn try_from(ptr: &*const oombak_parser_sys::Instance) -> OombakResult<InstanceNode> {
        let instance = unsafe { deref_instance_ptr(ptr)? };
        let name = string_from_ptr(instance.name)?;
        let module_name = string_from_ptr(instance.module_name)?;
        let signals = signals_ptr_to_vec(instance.signals, instance.signals_len as usize)?;
        let children = child_instances_ptr_to_vec(
            instance.child_instances,
            instance.child_instances_len as usize,
        )?;
        Ok(InstanceNode {
            name,
            module_name,
            signals,
            children,
        })
    }
}

impl TryFrom<&oombak_parser_sys::Signal> for Signal {
    type Error = crate::Error;

    fn try_from(value: &oombak_parser_sys::Signal) -> Result<Self, Self::Error> {
        let name = string_from_ptr(value.name)?;
        let width = value.width as usize;
        let signal_type = match value.signal_type {
            oombak_parser_sys::SignalType::PackedArrPortIn => {
                SignalType::PackedArrPort(Direction::In, width)
            }
            oombak_parser_sys::SignalType::PackedArrPortOut => {
                SignalType::PackedArrPort(Direction::Out, width)
            }
            oombak_parser_sys::SignalType::PackedArrVarNet => SignalType::PackedArrNetVar(width),
        };
        Ok(Signal { name, signal_type })
    }
}

impl Signal {
    pub fn is_port(&self) -> bool {
        match &self.signal_type {
            SignalType::PackedArrPort(_, _) => true,
            SignalType::PackedArrNetVar(_) => false,
        }
    }

    pub fn is_input_port(&self) -> bool {
        matches!(
            &self.signal_type,
            SignalType::PackedArrPort(Direction::In, _)
        )
    }

    pub fn bit_width(&self) -> usize {
        match &self.signal_type {
            SignalType::PackedArrPort(_, bit_width) => *bit_width,
            SignalType::PackedArrNetVar(bit_width) => *bit_width,
        }
    }
}

impl Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalType::PackedArrPort(Direction::In, _) => {
                write!(f, "unpacked array (input port)")
            }
            SignalType::PackedArrPort(Direction::Out, _) => {
                write!(f, "unpacked array (output port)")
            }
            SignalType::PackedArrNetVar(_) => write!(f, "unpacked array (net / var)"),
        }
    }
}

fn string_from_ptr(ptr: *const c_char) -> OombakResult<String> {
    if ptr.is_null() {
        return Err(Error::NullDereference.into());
    }
    Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_string())
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
    unsafe { std::slice::from_raw_parts(signals, signals_len) }
        .iter()
        .map(Signal::try_from)
        .collect()
}

fn child_instances_ptr_to_vec(
    child_instances: *const *const oombak_parser_sys::Instance,
    child_instances_len: usize,
) -> OombakResult<Vec<InstanceNode>> {
    if child_instances.is_null() {
        return Err(Error::NullDereference.into());
    }
    unsafe { std::slice::from_raw_parts(child_instances, child_instances_len) }
        .iter()
        .map(InstanceNode::try_from)
        .collect()
}

#[cfg(test)]
mod test {
    use super::{oombak_parser_sys::Instance, parse, Direction, InstanceNode, Signal, SignalType};
    use std::sync::OnceLock;

    static FIXTURES_PATH: OnceLock<String> = OnceLock::new();

    fn fixtures_path() -> &'static String {
        FIXTURES_PATH.get_or_init(|| {
            String::from(env!("CARGO_MANIFEST_DIR")) + "/oombak_parser/tests/fixtures"
        })
    }

    #[test]
    fn test_get_signal() {
        let mut root = InstanceNode {
            name: "root".to_string(),
            ..Default::default()
        };

        let child_0 = InstanceNode {
            name: "child_0".to_string(),
            ..Default::default()
        };

        let mut child_1 = InstanceNode {
            name: "child_1".to_string(),
            ..Default::default()
        };

        let child_2 = InstanceNode {
            name: "child_2".to_string(),
            signals: vec![
                Signal {
                    name: "sig_0".to_string(),
                    signal_type: SignalType::PackedArrNetVar(1),
                },
                Signal {
                    name: "sig_1".to_string(),
                    signal_type: SignalType::PackedArrNetVar(1),
                },
            ],
            ..Default::default()
        };

        child_1.children.push(child_2);
        root.children = vec![child_0, child_1];

        assert!(root
            .get_signal("root.child_1.child_2.sig_1")
            .unwrap()
            .is_some())
    }

    #[test]
    fn test_parse() {
        let source_paths = [
            format!("{}/sv_sample_1/adder.sv", fixtures_path()),
            format!("{}/sv_sample_1/sample.sv", fixtures_path()),
        ];
        let root = parse(&source_paths, "sample").unwrap();
        assert_eq!(root.name, "sample");
        assert_eq!(root.module_name, "sample");

        assert_eq!(root.signals.len(), 5);
        assert!(root.signals.contains(&Signal {
            name: "clk".to_string(),
            signal_type: SignalType::PackedArrPort(Direction::In, 1)
        }));
        assert!(root.signals.contains(&Signal {
            name: "rst_n".to_string(),
            signal_type: SignalType::PackedArrPort(Direction::In, 1)
        }));
        assert!(root.signals.contains(&Signal {
            name: "in".to_string(),
            signal_type: SignalType::PackedArrPort(Direction::In, 6)
        }));
        assert!(root.signals.contains(&Signal {
            name: "out".to_string(),
            signal_type: SignalType::PackedArrPort(Direction::Out, 6)
        }));
        assert!(root.signals.contains(&Signal {
            name: "c".to_string(),
            signal_type: SignalType::PackedArrNetVar(6)
        }));

        assert_eq!(root.children.len(), 1);
        let child = &root.children[0];

        assert_eq!(child.name, "adder_inst");
        assert_eq!(child.module_name, "adder");
        assert_eq!(child.children.len(), 0);
        assert_eq!(child.signals.len(), 4);
        assert!(child.signals.contains(&Signal {
            name: "a".to_string(),
            signal_type: SignalType::PackedArrPort(Direction::In, 6)
        }));
        assert!(child.signals.contains(&Signal {
            name: "b".to_string(),
            signal_type: SignalType::PackedArrPort(Direction::In, 6)
        }));
        assert!(child.signals.contains(&Signal {
            name: "c".to_string(),
            signal_type: SignalType::PackedArrPort(Direction::Out, 6)
        }));
        assert!(child.signals.contains(&Signal {
            name: "d".to_string(),
            signal_type: SignalType::PackedArrNetVar(1)
        }));
    }

    #[test]
    fn test_invalid_module() {
        let source_paths = [
            format!("{}/sv_sample_1/adder.sv", fixtures_path()),
            format!("{}/sv_sample_1/sample.sv", fixtures_path()),
        ];
        let e = parse(&source_paths, "invalid_module").unwrap_err();
        assert_eq!(
            &e.to_string(),
            "oombak_rs: probe: parse: top-level module not found"
        );
    }

    #[test]
    fn test_syntax_error() {
        let source_paths = [format!("{}/syntax_error/sample.sv", fixtures_path())];
        let e = parse(&source_paths, "sample").unwrap_err();
        assert_eq!(&e.to_string(), "oombak_rs: probe: parse: failed to compile:\noombak_parser/tests/fixtures/syntax_error/sample.sv:9:3: error: use of undeclared identifier 'ire'\n  ire d;\n  ^~~\n");
    }

    #[test]
    fn test_inout_port() {
        let source_paths = [format!("{}/inout_port/sample.sv", fixtures_path())];
        let e = parse(&source_paths, "sample").unwrap_err();
        assert_eq!(
            &e.to_string(),
            "oombak_rs: probe: parse: found unsupported port direction"
        );
    }

    #[test]
    fn test_unpacked_array() {
        let source_paths = [format!("{}/unpacked_array/sample.sv", fixtures_path())];
        let e = parse(&source_paths, "sample").unwrap_err();
        assert_eq!(
            &e.to_string(),
            "oombak_rs: probe: parse: found unsupported symbol type"
        );
    }

    #[test]
    fn test_file_not_found() {
        let source_paths = [format!("{}/invalid_folder/sample.sv", fixtures_path())];
        let e = parse(&source_paths, "sample").unwrap_err();
        assert_eq!(&e.to_string(), "oombak_rs: probe: parse: file not found");
    }

    #[test]
    fn test_null() {
        let ptr = std::ptr::null::<Instance>();
        let e = InstanceNode::try_from(&ptr).unwrap_err();
        assert_eq!(&e.to_string(), "oombak_rs: probe: parse: null dereference");
    }
}
