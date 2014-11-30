extern crate serialize;
extern crate regex;

use std::io::File;
use std::io::IoError;
use serialize::json;
use errors::InitializationError;
use serialize::json::DecoderError;
use serialize::json::ParserError;

#[deriving(Decodable)]
pub struct ClientConfiguration {
    pub nick: String,
    pub user: String,
    pub real_name: String,
    pub channels: Vec<String>,
    pub address: String,
    pub command_prefix: String,
    pub admins: Vec<String>
}

pub fn load_config_from_file(path: &Path) -> Result<ClientConfiguration, InitializationError> {
    let config_contents = try!(File::open(path).read_to_string());
    let client_config = try!(match json::decode::<ClientConfiguration>(config_contents.as_slice()) {
        Ok(v) => Ok(v),
        Err(e) => {
            match e {
                DecoderError::ParseError(parse_error) => match parse_error {
                    ParserError::SyntaxError(error_code, line, col) => return Err(InitializationError::from_string(format!("Syntax error ({}) on line {} column {} in {}", error_code, line, col, path.display()))),
                    ParserError::IoError(kind, desc) => return Err(InitializationError::Io(IoError{ kind: kind, desc: desc, detail: None}))
                },
                DecoderError::MissingFieldError(s) => return Err(InitializationError::from_string(format!("Field {} not found in {}", s.as_slice(), path.display()))),
                _ => Err(e)
            }
        }
    });

    return Ok(client_config)
}
