use thiserror::Error;

pub type DutGenResult<T> = Result<T, DutGenError>;

#[derive(Error, Debug)]
pub enum DutGenError {
    #[error("IO error: {}", _0)]
    Io(std::io::Error)
}

impl From<std::io::Error> for DutGenError {
    fn from(value: std::io::Error) -> Self {
        DutGenError::Io(value)
    }
}

impl From<DutGenError> for String {
    fn from(value: DutGenError) -> Self {
        value.to_string()
    }
}
