#![feature(plugin)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate chrono;
extern crate regex;
extern crate fern;
#[plugin]
extern crate regex_macros;
#[macro_use] #[no_link]
extern crate fern_macros;

pub use errors::InitializationError;
pub use config::ClientConfiguration;
pub use interface::IrcInterface;
pub use events::{
    MessageEvent,
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
mod dispatch;
mod events;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
