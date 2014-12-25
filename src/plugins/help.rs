use std::io::timer;
use std::time::duration;
use std::rand;

use client::PluginRegister;
use events::CommandEvent;

fn help(event: &CommandEvent) {
    let plugins = event.client.plugins.read();
    let commands = plugins.commands.keys().collect::<Vec<&String>>().connect(", ");
    event.client.send_message(event.channel(), format!("Available commands: {}", commands).as_slice());
    event.client.send_message(event.channel(), format!("Use a command with `{}command_name` or `{}, command_name`", event.client.command_prefix, event.client.state.read().nick).as_slice());
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("help", help);
}
