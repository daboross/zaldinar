use client::PluginRegister;

mod tracker;
mod log;
mod ctcp;
mod help;

pub fn register_plugins(register: &mut PluginRegister) {
    tracker::register(register);
    log::register(register);
    ctcp::register(register);
    help::register(register);
}

// TODO: Implement commands from http://sprunge.us/KSSH
