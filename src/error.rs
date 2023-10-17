use std::io;

#[derive(Debug)]
pub enum EmulatorError {
    FileError(io::Error),
}

impl From<std::io::Error> for EmulatorError {
    fn from(value: std::io::Error) -> Self {
        EmulatorError::FileError(value)
    }
}