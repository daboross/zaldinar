#![feature(plugin)] // For regex_macros
#![feature(std_misc)] // For hash_map::Entry
#![plugin(regex_macros)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate regex;
#[macro_use]
extern crate log;
extern crate fern;
extern crate "zaldinar-irclib" as irc;

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
