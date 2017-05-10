extern crate chrono;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate fern;
#[macro_use]
extern crate throw;

#[cfg(feature = "binary-filewatch")]
extern crate inotify;

extern crate zaldinar_irclib as irc;
extern crate zaldinar_core as core;
extern crate generated_plugins_crate;

macro_rules! regex {
    ($s:expr) => ({
        lazy_static! {
            static ref REGEX: ::regex::Regex = ::regex::Regex::new("...").unwrap();
        }
        &REGEX
    })
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

pub mod startup;
pub mod dispatch;
mod plugins;
#[cfg(feature = "binary-filewatch")]
mod filewatch;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
