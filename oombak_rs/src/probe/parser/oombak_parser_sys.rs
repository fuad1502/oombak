use std::ffi::{c_char, c_void};

#[link(name = "oombak_parser")]
extern "C" {
    pub fn oombak_parser_parse_r(
        ctx: Context,
        source_paths: *const c_char,
        top_level_module_name: *const c_char,
    ) -> Result;

    pub fn oombak_parser_get_ctx() -> Context;

    pub fn oombak_parser_free_ctx(ctx: Context);
}

type Context = *const c_void;

#[repr(C)]
pub struct Result {
    pub is_error: u8,
    pub instance_or_error: InstanceOrError,
}

#[repr(C)]
pub union InstanceOrError {
    pub instance: *const Instance,
    pub error: Error,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    None,
    FileNotFound,
    TopLevelModuleNotFound,
    CompileError,
    UnsupportedSymbolType,
    UnsupportedPortDirection,
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
    PackedArrPortIn,
    PackedArrPortOut,
    PackedArrVarNet,
}
