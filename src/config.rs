use std::io;
use rustc_serialize::json;

use errors::InitializationError;

#[deriving(RustcDecodable)]
pub struct NickServConf {
    pub name: String,
    pub command: String,
    pub account: String,
    pub password: String,
    pub enabled: bool,
}

#[deriving(RustcDecodable)]
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
}

impl ClientConfiguration {
    pub fn load_from_file(path: &Path) -> Result<ClientConfiguration, InitializationError> {
        let config_contents = try!(io::File::open(path).read_to_string());
        let client_config = try!(match json::decode::<ClientConfiguration>(config_contents.as_slice()) {
            Ok(v) => Ok(v),
            Err(e) => {
                match e {
                    json::DecoderError::ParseError(parse_error) => match parse_error {
                        json::ParserError::SyntaxError(error_code, line, col) => return Err(InitializationError::from_string(format!("Syntax error ({}) on line {} column {} in {}", error_code, line, col, path.display()))),
                        json::ParserError::IoError(kind, desc) => return Err(InitializationError::Io(io::IoError{ kind: kind, desc: desc, detail: None})),
                    },
                    json::DecoderError::MissingFieldError(s) => return Err(InitializationError::from_string(format!("Field {} not found in {}", s.as_slice(), path.display()))),
                    _ => Err(e),
                }
            },
        });

        return Ok(client_config);
    }
}
