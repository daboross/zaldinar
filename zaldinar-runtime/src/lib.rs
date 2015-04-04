extern crate rustc_serialize;
extern crate time;
extern crate regex;
#[macro_use]
extern crate log;
extern crate fern;
extern crate zaldinar_irclib as irc;
extern crate zaldinar_core as core;
extern crate generated_plugins_crate;

macro_rules! regex {
    ($s:expr) => (::regex::Regex::new($s).unwrap())
}

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
#[cfg(feature = "binary-filewatch")]
mod filewatch;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
