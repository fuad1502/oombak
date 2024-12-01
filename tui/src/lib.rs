use libloading::{Library, Symbol};
use std::ffi::{c_char, c_int, CString};
use bitvec::vec::BitVec;

#[derive(Debug)]
pub struct Error {
    _message: String,
}

impl From<libloading::Error> for Error {
    fn from(value: libloading::Error) -> Self {
        Self {
            _message: value.to_string(),
        }
    }
}

pub struct Dut {
    lib: DutLib,
}

impl Dut {
    pub fn new(lib_path: &str) -> Result<Self, Error> {
        let lib = DutLib::new(lib_path)?;
        Ok(Dut { lib })
    }

    pub fn run(&self, duration: u64) -> Result<u64, Error> {
        let current_time: u64 = 0;
        match self.lib.run(duration, &current_time)? {
            0 => Ok(current_time),
            _ => Err(Error {
                _message: "failed to run".to_string(),
            }),
        }
    }

    pub fn set(&self, sig_name: &str, bit_vec: &BitVec<u32>) -> Result<(), Error> {
        let c_str = CString::new(sig_name).unwrap();
        let words = bit_vec.as_raw_slice();
        match self
            .lib
            .set(c_str.as_ptr(), words.as_ptr(), words.len() as u64)?
        {
            0 => Ok(()),
            _ => Err(Error {
                _message: "failed to set".to_string(),
            }),
        }
    }

    pub fn get(&self, sig_name: &str) -> Result<BitVec<u32>, Error> {
        let c_str = CString::new(sig_name).unwrap();
        let mut len: u64 = 0;
        let res = self.lib.get(c_str.as_ptr(), &mut len as *mut u64)?;
        if res == std::ptr::null() {
            return Err(Error {
                _message: "failed to get".to_string(),
            });
        }
        let slice_len = len as usize / 32 + if len % 32 != 0 { 1 } else { 0 };
        let slice = unsafe { std::slice::from_raw_parts(res, slice_len) };
        let mut bit_vec = BitVec::from_slice(slice);
        bit_vec.truncate(len as usize);
        Ok(bit_vec)
    }
}

struct DutLib {
    lib: Library,
}

impl DutLib {
    fn new(lib_path: &str) -> Result<Self, libloading::Error> {
        let lib = unsafe { Library::new(lib_path)? };
        Ok(DutLib { lib })
    }

    fn run(&self, duration: u64, current_time_o: *const u64) -> Result<c_int, libloading::Error> {
        let f: Symbol<unsafe extern "C" fn(u64, *const u64) -> c_int> =
            unsafe { self.lib.get(b"run")? };
        Ok(unsafe { f(duration, current_time_o) })
    }

    fn set(
        &self,
        sig_name: *const c_char,
        bytes: *const u32,
        len: u64,
    ) -> Result<c_int, libloading::Error> {
        let f: Symbol<unsafe extern "C" fn(*const c_char, *const u32, u64) -> c_int> =
            unsafe { self.lib.get(b"set")? };
        Ok(unsafe { f(sig_name, bytes, len) })
    }

    fn get(&self, sig_name: *const c_char, len: *mut u64) -> Result<*const u32, libloading::Error> {
        let f: Symbol<unsafe extern "C" fn(*const c_char, *mut u64) -> *mut u32> =
            unsafe { self.lib.get(b"get")? };
        Ok(unsafe { f(sig_name, len) })
    }
}
