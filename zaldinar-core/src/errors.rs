use std::error;
use std::io;
use std::sync;
use std::fmt;

use rustc_serialize::json;
use regex;
use fern;

#[derive(Debug)]
pub enum InitializationError {
    Io(io::Error),
    Regex(regex::Error),
    Decoder(json::DecoderError),
    FernInit(fern::InitError),
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

impl error::FromError<io::Error> for InitializationError {
    fn from_error(error: io::Error) -> InitializationError {
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
        InitializationError::Generic(format!("{}", error))
    }
}

impl error::FromError<fern::InitError> for InitializationError {
    fn from_error(error: fern::InitError) -> InitializationError {
        InitializationError::FernInit(error)
    }
}

impl error::FromError<String> for InitializationError {
    fn from_error(error: String) -> InitializationError {
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
            &InitializationError::FernInit(ref e) => {
                try!(fmt.write_str("Fern Initialization Error: "));
                fmt::Display::fmt(e, fmt)
            },
            &InitializationError::Generic(ref s) => {
                try!(fmt.write_str("Error: "));
                fmt::Display::fmt(s, fmt)
            }
        }
    }
}
