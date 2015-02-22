#![feature(plugin, box_syntax, core, std_misc, io, path, old_io, old_path, collections)]
#![cfg_attr(target_os = "linux", feature(env, os))] // for filewatch
#![plugin(regex_macros)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate chrono;
extern crate regex;
extern crate fern;
extern crate inotify;
extern crate rand;
#[macro_use]
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
#[cfg(target_os = "linux")]
mod filewatch;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
