extern crate "zaldinar-core" as zaldinar;

use zaldinar::client::PluginRegister;
use zaldinar::events::CommandEvent;

fn info_command(event: &CommandEvent) {
    event.client.send_message(event.channel(), format!("Hi, I'm zaldinar version {} - \
        created by Dabo - Powered by Rust!", zaldinar::VERSION));
    event.client.send_message(event.channel(), "Source code available at https://github.com/\
        daboross/zaldinar/");
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("info", info_command);
}
