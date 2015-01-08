use std::error;
use std::io;
use std::sync;
use rustc_serialize::json;
use regex;

#[derive(Show)]
pub enum InitializationError {
    Io(io::IoError),
    Regex(regex::Error),
    Decoder(json::DecoderError),
    Generic(String),
}

impl InitializationError {
    pub fn new(detail: &str) -> InitializationError {
        InitializationError::Generic(detail.to_string())
    }

    pub fn from_string(detail: String) -> InitializationError {
        InitializationError::Generic(detail)
    }
}

impl error::FromError<io::IoError> for InitializationError {
    fn from_error(error: io::IoError) -> InitializationError {
        InitializationError::Io(error)
    }
}

impl error::FromError<regex::Error> for InitializationError {
    fn from_error(error: regex::Error) -> InitializationError {
        InitializationError::Regex(error)
    }
}

impl error::FromError<json::DecoderError> for InitializationError {
    fn from_error(error: json::DecoderError) -> InitializationError {
        InitializationError::Decoder(error)
    }
}

impl <T> error::FromError<sync::PoisonError<T>> for InitializationError {
    fn from_error(error: sync::PoisonError<T>) -> InitializationError {
        InitializationError::Generic(format!("{}",error))
    }
}
