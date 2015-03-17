use std::io::prelude::*;
use std::fs;
use rustc_serialize::json;
use std::path::Path;

use errors::InitializationError;

#[derive(RustcDecodable)]
pub struct NickServConf {
    pub name: String,
    pub command: String,
    pub account: String,
    pub password: String,
    pub enabled: bool,
}

#[derive(RustcDecodable)]
pub struct ClientConfiguration {
    pub nick: String,
    pub user: String,
    pub real_name: String,
    pub address: String,
    pub nickserv: NickServConf,
    pub channels: Vec<String>,
    pub command_prefix: String,
    pub admins: Vec<String>,
    pub on_connect: Vec<String>,
    pub password: Option<String>,
    pub log_file: String,
    pub watch_binary: bool,
}

impl ClientConfiguration {
    pub fn load_from_file(path: &Path) -> Result<ClientConfiguration, InitializationError> {
        let config_contents = {
            let mut buf = String::new();
            try!(try!(fs::File::open(path)).read_to_string(&mut buf));
            buf
        };
        return Ok(try!(match json::decode::<ClientConfiguration>(&config_contents) {
            Err(json::DecoderError::MissingFieldError(s)) => {
                return Err(InitializationError::from_string(format!("Field {} not found in {}",
                    &s, path.display())));
            },
            Err(json::DecoderError::ParseError(
                    json::ParserError::SyntaxError(error_code, line, col))) => {
                return Err(InitializationError::from_string(format!(
                    "Syntax error ({:?}) on line {} column {} in {}",
                    error_code, line, col, path.display())));
            },
            Err(json::DecoderError::ParseError(json::ParserError::IoError(err))) => {
                return Err(InitializationError::Io(err));
            },
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }));
    }
}
