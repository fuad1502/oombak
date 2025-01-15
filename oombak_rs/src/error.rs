use crate::{dut, parser, probe};

pub type OombakResult<T> = Result<T, Box<dyn OombakError>>;

pub trait OombakError {
    fn kind(&self) -> &ErrorKind;
    fn inner(&self) -> &dyn std::error::Error;
}

impl std::fmt::Display for Box<dyn OombakError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ombak: {}: {}", self.kind(), self.inner())
    }
}

impl std::fmt::Debug for Box<dyn OombakError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ kind: {:?}, inner: {:?} }}",
            self.kind(),
            self.inner()
        )
    }
}

impl std::error::Error for Box<dyn OombakError> {}

#[derive(Debug)]
pub enum ErrorKind {
    LibLoading,
    Dut,
    Parse,
    Probe,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::LibLoading => write!(f, "libloading"),
            ErrorKind::Dut => write!(f, "dut"),
            ErrorKind::Parse => write!(f, "parse"),
            ErrorKind::Probe => write!(f, "probe"),
        }
    }
}

struct SimpleOmbakError {
    kind: ErrorKind,
    inner: Box<dyn std::error::Error>,
}

impl OombakError for SimpleOmbakError {
    fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    fn inner(&self) -> &dyn std::error::Error {
        &*self.inner
    }
}

impl From<libloading::Error> for Box<dyn OombakError> {
    fn from(error: libloading::Error) -> Self {
        Box::new(SimpleOmbakError {
            kind: ErrorKind::LibLoading,
            inner: Box::new(error),
        })
    }
}

impl From<dut::DutError> for Box<dyn OombakError> {
    fn from(error: dut::DutError) -> Self {
        Box::new(SimpleOmbakError {
            kind: ErrorKind::Dut,
            inner: Box::new(error),
        })
    }
}

impl From<parser::ParseError> for Box<dyn OombakError> {
    fn from(error: parser::ParseError) -> Self {
        Box::new(SimpleOmbakError {
            kind: ErrorKind::Parse,
            inner: Box::new(error),
        })
    }
}

impl From<probe::ProbeError> for Box<dyn OombakError> {
    fn from(error: probe::ProbeError) -> Self {
        Box::new(SimpleOmbakError {
            kind: ErrorKind::Probe,
            inner: Box::new(error),
        })
    }
}

impl From<Box<dyn OombakError>> for String {
    fn from(error: Box<dyn OombakError>) -> Self {
        error.to_string()
    }
}
