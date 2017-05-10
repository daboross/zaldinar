use std::io;
use std::sync;
use std::fmt;
use serde_json;
use regex;
use throw;

pub type ThrowInitError = throw::Error<InitializationError>;

#[derive(Debug)]
pub enum InitializationError {
    Io(io::Error),
    Regex(regex::Error),
    Decoder(serde_json::Error),
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

impl From<io::Error> for InitializationError {
    fn from(error: io::Error) -> InitializationError {
        InitializationError::Io(error)
    }
}

impl From<regex::Error> for InitializationError {
    fn from(error: regex::Error) -> InitializationError {
        InitializationError::Regex(error)
    }
}

impl From<serde_json::Error> for InitializationError {
    fn from(error: serde_json::Error) -> InitializationError {
        InitializationError::Decoder(error)
    }
}

impl <T> From<sync::PoisonError<T>> for InitializationError {
    fn from(error: sync::PoisonError<T>) -> InitializationError {
        InitializationError::Generic(format!("{}", error))
    }
}

impl From<String> for InitializationError {
    fn from(error: String) -> InitializationError {
        InitializationError::Generic(error)
    }
}

impl fmt::Display for InitializationError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &InitializationError::Io(ref e) => {
                try!(fmt.write_str("IO Error: "));
                fmt::Display::fmt(e, fmt)
            },
            &InitializationError::Regex(ref e) => {
                try!(fmt.write_str("Regex Error: "));
                fmt::Display::fmt(e, fmt)
            },
            &InitializationError::Decoder(ref e) => {
                try!(fmt.write_str("JSON Error: "));
                fmt::Display::fmt(e, fmt)
            },
            &InitializationError::Generic(ref s) => {
                try!(fmt.write_str("Error: "));
                fmt::Display::fmt(s, fmt)
            }
        }
    }
}
