#![feature(phase)]
#![feature(unboxed_closures)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate chrono;
extern crate regex;
extern crate fern;
#[phase(plugin)] extern crate regex_macros;
#[phase(plugin, link)] extern crate fern_macros;

pub use errors::InitializationError;
pub use config::ClientConfiguration;
pub use interface::{
    IrcInterface,
    IrcMessageEvent,
    CommandEvent,
    CtcpEvent,
};
pub use client::run;
pub use client::run_with_plugins;

pub mod errors;
pub mod config;
pub mod interface;
pub mod client;
mod irc;
mod plugins;

pub fn get_version() -> String {
    return format!("{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));
}

