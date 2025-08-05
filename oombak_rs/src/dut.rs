use bitvec::vec::BitVec;
use dut_sys::SigT;
use std::ffi::{CStr, CString};

use crate::OombakResult;

mod dut_sys;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to query signals")]
    Query,
    #[error("failed to run")]
    Run,
    #[error("failed to set signal {} with value {}", _0, _1)]
    Set(String, BitVec<u32>),
    #[error("failed to get signal {}", _0)]
    Get(String),
    #[error("libloading: {}", _0)]
    Libloading(libloading::Error),
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Dut(value)
    }
}

impl From<libloading::Error> for crate::Error {
    fn from(value: libloading::Error) -> Self {
        Error::Libloading(value).into()
    }
}

pub struct Dut {
    lib: dut_sys::DutLib,
}

impl Dut {
    pub fn new(lib_path: &str) -> OombakResult<Self> {
        let lib = dut_sys::DutLib::new(lib_path)?;
        Ok(Dut { lib })
    }

    pub fn query(&self) -> OombakResult<Vec<Signal>> {
        let mut num_of_signals: u64 = 0;
        let sig_t_ptr = self.lib.query(&mut num_of_signals as *mut u64)?;
        Ok(Self::signals_from(sig_t_ptr, num_of_signals as usize))
    }

    pub fn run(&self, duration: u64) -> OombakResult<u64> {
        let current_time: u64 = 0;
        match self.lib.run(duration, &current_time)? {
            0 => Ok(current_time),
            _ => Err(Error::Run.into()),
        }
    }

    pub fn set(&self, sig_name: &str, bit_vec: &BitVec<u32>) -> OombakResult<()> {
        let c_str = CString::new(sig_name)?;
        let words = bit_vec.as_raw_slice();
        match self
            .lib
            .set(c_str.as_ptr(), words.as_ptr(), words.len() as u64)?
        {
            0 => Ok(()),
            _ => Err(Error::Set(sig_name.to_string(), bit_vec.clone()).into()),
        }
    }

    pub fn get(&self, sig_name: &str) -> OombakResult<BitVec<u32>> {
        let sig_name_cstr = CString::new(sig_name)?;
        let mut n_bits: u64 = 0;
        let words_ptr = self
            .lib
            .get(sig_name_cstr.as_ptr(), &mut n_bits as *mut u64)?;
        if words_ptr.is_null() {
            return Err(Error::Get(sig_name.to_string()).into());
        }
        Ok(Self::bitvec_from(words_ptr, n_bits as usize))
    }

    fn bitvec_from(words_ptr: *const u32, n_bits: usize) -> BitVec<u32> {
        let num_of_words = n_bits / 32 + if n_bits % 32 != 0 { 1 } else { 0 };
        let slice = unsafe { std::slice::from_raw_parts(words_ptr, num_of_words) };
        let mut bit_vec = BitVec::from_slice(slice);
        bit_vec.truncate(n_bits);
        bit_vec
    }

    fn signals_from(sig_t_ptr: *const dut_sys::SigT, num_of_signals: usize) -> Vec<Signal> {
        let sig_t_slice = unsafe { std::slice::from_raw_parts(sig_t_ptr, num_of_signals) };
        sig_t_slice.iter().map(Signal::from).collect()
    }
}

#[derive(Debug)]
pub struct Signal {
    pub name: String,
    pub width: u64,
    pub get: bool,
    pub set: bool,
}

impl From<&SigT> for Signal {
    fn from(value: &SigT) -> Self {
        let name =
            String::from_utf8_lossy((unsafe { CStr::from_ptr(value.name) }).to_bytes()).to_string();
        let get = value.get == 1;
        let set = value.set == 1;
        Signal {
            name,
            width: value.width,
            get,
            set,
        }
    }
}
