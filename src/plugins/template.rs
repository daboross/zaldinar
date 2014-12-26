use client::PluginRegister;
use events::CommandEvent;


pub fn register(register: &mut PluginRegister) {
    register.register_command();
}
