pub type OombakSimResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DUT not loaded")]
    DutNotLoaded,
    #[error("DUT is loading")]
    DutIsLoading,
    #[error("oombak_gen: {}", _0)]
    OombakGen(oombak_gen::Error),
    #[error("oombak_rs: {}", _0)]
    Oombak(oombak_rs::Error),
}

impl From<oombak_gen::Error> for Error {
    fn from(value: oombak_gen::Error) -> Self {
        Self::OombakGen(value)
    }
}

impl From<oombak_rs::Error> for Error {
    fn from(value: oombak_rs::Error) -> Self {
        Self::Oombak(value)
    }
}
