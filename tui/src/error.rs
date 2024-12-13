use crate::dut;

pub type OmbakResult<T> = Result<T, Box<dyn OmbakError>>;

pub trait OmbakError {
    fn kind(&self) -> &ErrorKind;
    fn inner(&self) -> &Box<dyn std::error::Error>;
}

impl std::fmt::Display for Box<dyn OmbakError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ombak: {}: {}", self.kind(), self.inner())
    }
}

impl std::fmt::Debug for Box<dyn OmbakError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ kind: {:?}, inner: {:?} }}",
            self.kind(),
            self.inner()
        )
    }
}

impl std::error::Error for Box<dyn OmbakError> {}

#[derive(Debug)]
pub enum ErrorKind {
    LibLoading,
    Dut,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::LibLoading => write!(f, "libloading"),
            ErrorKind::Dut => write!(f, "dut"),
        }
    }
}

struct SimpleOmbakError {
    kind: ErrorKind,
    inner: Box<dyn std::error::Error>,
}

impl OmbakError for SimpleOmbakError {
    fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    fn inner(&self) -> &Box<dyn std::error::Error> {
        &self.inner
    }
}

impl From<libloading::Error> for Box<dyn OmbakError> {
    fn from(error: libloading::Error) -> Self {
        Box::new(SimpleOmbakError {
            kind: ErrorKind::LibLoading,
            inner: Box::new(error),
        })
    }
}

impl From<dut::DutError> for Box<dyn OmbakError> {
    fn from(error: dut::DutError) -> Self {
        Box::new(SimpleOmbakError {
            kind: ErrorKind::Dut,
            inner: Box::new(error),
        })
    }
}
