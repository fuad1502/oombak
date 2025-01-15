use std::ffi::c_char;

#[link(name = "oombak_parser")]
extern "C" {
    pub fn oombak_parser_parse(
        source_paths: *const c_char,
        top_module_name: *const c_char,
    ) -> *const Instance;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Instance {
    pub name: *const c_char,
    pub module_name: *const c_char,
    pub parent_instance: *const Instance,
    pub child_instances: *const *const Instance,
    pub child_instances_len: u64,
    pub signals: *const Signal,
    pub signals_len: u64,
}

#[repr(C)]
pub struct Signal {
    pub name: *const c_char,
    pub signal_type: SignalType,
    pub width: u64,
}

#[repr(C)]
#[allow(clippy::enum_variant_names)]
pub enum SignalType {
    UnpackedArrPortIn,
    UnpackedArrPortOut,
    UnpackedArrVarNet,
}
