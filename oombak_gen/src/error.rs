use std::path::PathBuf;

pub type OombakGenResult<T> = Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(".sv file path not found: {}", _0.to_string_lossy())]
    SvFilePathNotFound(PathBuf),
    #[error("invalid path given: {}", _0.to_string_lossy())]
    InvalidPath(PathBuf),
    #[error("file name does not have .sv extension: {}", _0.to_string_lossy())]
    ExtensionNotSv(PathBuf),
    #[error("IO error: {}", _0)]
    Io(std::io::Error),
    #[error("CMake error: {}", _0)]
    CMake(String),
    #[error("oombak_rs: {}", _0)]
    Oombak(oombak_rs::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<oombak_rs::Error> for Error {
    fn from(value: oombak_rs::Error) -> Self {
        Error::Oombak(value)
    }
}
