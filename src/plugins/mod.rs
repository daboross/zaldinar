use client::Client;

mod control;
mod log;
mod ctcp;

pub fn register_plugins(client: &mut Client) {
    control::register(client);
    log::register(client);
    ctcp::register(client);
}
