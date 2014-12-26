use client::PluginRegister;

mod tracker;
mod control;
mod log;
mod ctcp;
mod help;
mod eightball;
mod choose;

pub fn register_plugins(register: &mut PluginRegister) {
    tracker::register(register);
    control::register(register);
    log::register(register);
    ctcp::register(register);
    help::register(register);
    eightball::register(register);
    choose::register(register);
}

// TODO: Implement commands from http://sprunge.us/KSSH
