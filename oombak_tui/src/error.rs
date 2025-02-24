use thiserror::Error;

pub type OombakTuiResult<T> = Result<T, OombakTuiError>;

#[derive(Error, Debug)]
pub enum OombakTuiError {
    #[error("oombak_tui: IO error: _0")]
    IoError(std::io::Error),
}

impl From<std::io::Error> for OombakTuiError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}
