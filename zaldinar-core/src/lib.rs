extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate regex;
#[macro_use]
extern crate log;
#[macro_use]
extern crate throw;
extern crate zaldinar_irclib as irc;

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
