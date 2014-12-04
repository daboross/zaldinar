extern crate serialize;
extern crate regex;

use std::fmt;
use std::error;
use std::io::IoError;
use serialize::json::DecoderError;

pub enum InitializationError {
    Io(IoError),
    Regex(regex::Error),
    Decoder(DecoderError),
    Other(String)
}

impl InitializationError {
    pub fn new(detail: &str) -> InitializationError {
        InitializationError::Other(detail.into_string())
    }

    pub fn from_string(detail: String) -> InitializationError {
        InitializationError::Other(detail)
    }
}

impl error::FromError<IoError> for InitializationError {
    fn from_error(error: IoError) -> InitializationError {
        InitializationError::Io(error)
    }
}

impl error::FromError<regex::Error> for InitializationError {
    fn from_error(error: regex::Error) -> InitializationError {
        InitializationError::Regex(error)
    }
}

impl error::FromError<DecoderError> for InitializationError {
    fn from_error(error: DecoderError) -> InitializationError {
        InitializationError::Decoder(error)
    }
}

impl fmt::Show for InitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            InitializationError::Io(ref v) => v.fmt(formatter),
            InitializationError::Regex(ref v) => v.fmt(formatter),
            InitializationError::Decoder(ref v) => v.fmt(formatter),
            InitializationError::Other(ref v) => v.fmt(formatter),
        }
    }
}
