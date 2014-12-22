use client::PluginRegister;

mod control;
mod log;
mod ctcp;

pub fn register_plugins(register: &PluginRegister) {
    control::register(register);
    log::register(register);
    ctcp::register(register);
}
