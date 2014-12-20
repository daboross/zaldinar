#![feature(phase)]
#![feature(unboxed_closures)]

extern crate serialize;
extern crate chrono;
extern crate regex;
extern crate fern;
#[phase(plugin)] extern crate regex_macros;
#[phase(plugin, link)] extern crate fern_macros;

pub use errors::InitializationError;
pub use config::ClientConfiguration;
pub use client::Client;
pub use interface::{
    IrcInterface,
    IrcMessageEvent,
    CommandEvent,
    CtcpEvent,
};

pub mod errors;
pub mod config;
pub mod interface;
mod irc;
mod plugins;
mod client;

pub fn get_version() -> String {
    return format!("{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));
}

