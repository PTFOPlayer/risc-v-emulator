use std::{io, string::FromUtf8Error};

#[derive(Debug)]
pub enum EmulatorError {
    FileError(io::Error),
    FromUtf8(FromUtf8Error),
    StrTabError,
    WrongHeaderProvieded,
    NoTextSection,
}

impl From<std::io::Error> for EmulatorError {
    fn from(value: std::io::Error) -> Self {
        EmulatorError::FileError(value)
    }
}

impl From<FromUtf8Error> for EmulatorError {
    fn from(value: FromUtf8Error) -> Self {
        return EmulatorError::FromUtf8(value);
    }
}
