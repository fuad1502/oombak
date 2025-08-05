use crate::{dut, probe};

pub type OombakResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("oombak_rs: dut: {}", _0)]
    Dut(dut::Error),
    #[error("oombak_rs: probe: {}", _0)]
    Probe(probe::Error),
    #[error("oombak_rs: internal error: {}", _0)]
    InternalError(String),
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(value: std::sync::PoisonError<T>) -> Self {
        Error::InternalError(value.to_string())
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(value: std::ffi::NulError) -> Self {
        Error::InternalError(value.to_string())
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Error::InternalError(value.to_string())
    }
}
