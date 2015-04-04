extern crate rustc_serialize;
extern crate regex;
#[macro_use]
extern crate log;
extern crate fern;
extern crate zaldinar_irclib as irc;

macro_rules! regex {
    ($s:expr) => (::regex::Regex::new($s).unwrap())
}

pub use errors::InitializationError;
pub use config::ClientConfiguration;
pub use interface::IrcInterface;
pub use events::{
    MessageEvent,
    CommandEvent,
    CtcpEvent,
};

pub mod errors;
pub mod config;
pub mod interface;
pub mod client;
pub mod events;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
