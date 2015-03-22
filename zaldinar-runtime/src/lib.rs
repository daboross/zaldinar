#![feature(plugin)] // For regex_macros
#![cfg_attr(target_os = "linux", feature(libc))] // For filewatch - libc::nanosleep
#![plugin(regex_macros)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate chrono;
extern crate regex;
#[macro_use]
extern crate log;
extern crate fern;
extern crate inotify;
extern crate "zaldinar-irclib" as irc;
extern crate "zaldinar-core" as core;
extern crate "generated-plugins-crate" as generated_plugins_crate;

pub use core::config::ClientConfiguration;
pub use core::errors::InitializationError;
pub use core::errors;
pub use core::config;
pub use core::interface;
pub use core::client;
pub use core::events;
pub use startup::run;
pub use startup::run_with_plugins;

mod startup;
mod plugins;
mod dispatch;
#[cfg(target_os = "linux")]
mod filewatch;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
