use libloading::{Library, Symbol};
use std::ffi::{c_char, c_int};

use crate::error::OombakResult;

pub struct DutLib {
    lib: Library,
}

impl DutLib {
    pub fn new(lib_path: &str) -> OombakResult<Self> {
        let lib = unsafe { Library::new(lib_path)? };
        Ok(DutLib { lib })
    }

    pub fn query(&self, num_of_signals: *mut u64) -> OombakResult<*const OombakSigT> {
        let f: Symbol<unsafe extern "C" fn(*mut u64) -> *const OombakSigT> =
            unsafe { self.lib.get(b"oombak_query")? };
        Ok(unsafe { f(num_of_signals) })
    }

    pub fn run(&self, duration: u64, current_time_o: *const u64) -> OombakResult<c_int> {
        let f: Symbol<unsafe extern "C" fn(u64, *const u64) -> c_int> =
            unsafe { self.lib.get(b"oombak_run")? };
        Ok(unsafe { f(duration, current_time_o) })
    }

    pub fn set(
        &self,
        sig_name: *const c_char,
        words: *const u32,
        num_of_words: u64,
    ) -> OombakResult<c_int> {
        let f: Symbol<unsafe extern "C" fn(*const c_char, *const u32, u64) -> c_int> =
            unsafe { self.lib.get(b"oombak_set")? };
        Ok(unsafe { f(sig_name, words, num_of_words) })
    }

    pub fn get(&self, sig_name: *const c_char, n_bits: *mut u64) -> OombakResult<*const u32> {
        let f: Symbol<unsafe extern "C" fn(*const c_char, *mut u64) -> *mut u32> =
            unsafe { self.lib.get(b"oombak_get")? };
        Ok(unsafe { f(sig_name, n_bits) })
    }
}

#[repr(C)]
pub struct OombakSigT {
    pub name: *const c_char,
    pub width: u64,
    pub get: u8,
    pub set: u8,
}
