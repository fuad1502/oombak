use thiserror::Error;

pub type ThreadResult = Result<(), ThreadError>;

#[derive(Error, Debug)]
pub enum ThreadError {
    #[error("thread panicked: {}", _0)]
    Panic(String),
    #[error("IO error: {}", _0)]
    Io(std::io::Error),
}
