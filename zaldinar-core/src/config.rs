use std::io::prelude::*;
use std::fs;
use serde_json;
use std::path::Path;

use errors::ThrowInitError;

#[derive(Deserialize)]
pub struct NickServConf {
    pub name: String,
    pub command: String,
    pub account: String,
    pub password: String,
    pub enabled: bool,
}

#[derive(Deserialize)]
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
    pub log_level: String,
    pub watch_binary: bool,
}

impl ClientConfiguration {
    pub fn load_from_file(path: &Path) -> Result<ClientConfiguration, ThrowInitError> {
        let config_contents = {
            let mut buf = String::new();
            throw!(throw!(fs::File::open(path)).read_to_string(&mut buf));
            buf
        };

        Ok(throw!(serde_json::from_str(&config_contents)))
    }
}
