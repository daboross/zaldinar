extern crate "zaldinar-core" as zaldinar;

use zaldinar::client::PluginRegister;
use zaldinar::events::CommandEvent;

fn command(event: &CommandEvent) {

}

pub fn register(register: &mut PluginRegister) {
    register.register_command("", command);
}
