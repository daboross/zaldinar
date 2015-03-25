extern crate zaldinar_core;

use zaldinar_core::client::PluginRegister;
use zaldinar_core::events::CommandEvent;

fn command(event: &CommandEvent) {

}

pub fn register(register: &mut PluginRegister) {
    register.register_command("", command);
}
