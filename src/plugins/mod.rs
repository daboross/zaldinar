use Client;

pub mod control;
pub mod log;
pub mod ctcp;

pub fn register_plugins(client: &mut Client) {
    control::register(client);
    log::register(client);
    ctcp::register(client);
}
