use oombak_gen::error::OombakGenError;
use oombak_rs::error::OombakError;
use thiserror::Error;

pub type OombakSimResult<T> = Result<T, OombakSimError>;

#[derive(Debug, Error)]
pub enum OombakSimError {
    #[error("DUT not loaded")]
    DutNotLoaded,
    #[error("oombak_gen: {}", _0)]
    OombakGen(OombakGenError),
    #[error("oombak_rs: {}", _0)]
    Oombak(OombakError),
}

impl From<OombakGenError> for OombakSimError {
    fn from(value: OombakGenError) -> Self {
        Self::OombakGen(value)
    }
}

impl From<OombakError> for OombakSimError {
    fn from(value: OombakError) -> Self {
        Self::Oombak(value)
    }
}
