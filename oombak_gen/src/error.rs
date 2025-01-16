use std::path::PathBuf;

use oombak_rs::error::OombakError;
use thiserror::Error;

pub type OombakGenResult<T> = Result<T, OombakGenError>;

#[derive(Error, Debug)]
pub enum OombakGenError {
    #[error("invalid path given: {}", _0.to_string_lossy())]
    InvalidPath(PathBuf),
    #[error("IO error: {}", _0)]
    Io(std::io::Error),
    #[error("oombak_rs: {}", _0)]
    Oombak(OombakError),
}

impl From<std::io::Error> for OombakGenError {
    fn from(value: std::io::Error) -> Self {
        OombakGenError::Io(value)
    }
}

impl From<OombakError> for OombakGenError {
    fn from(value: OombakError) -> Self {
        OombakGenError::Oombak(value)
    }
}
