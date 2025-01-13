use thiserror::Error;

pub type OombakGenResult<T> = Result<T, OombakGenError>;

#[derive(Error, Debug)]
pub enum OombakGenError {
    #[error("IO error: {}", _0)]
    Io(std::io::Error),
}

impl From<std::io::Error> for OombakGenError {
    fn from(value: std::io::Error) -> Self {
        OombakGenError::Io(value)
    }
}

impl From<OombakGenError> for String {
    fn from(value: OombakGenError) -> Self {
        value.to_string()
    }
}
