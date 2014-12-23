use client;
use interface;

fn help(event: &interface::CommandEvent) {
    let plugins = event.client.plugins.read();
    let commands = plugins.commands.keys().collect::<Vec<&String>>().connect(", ");
    event.client.send_message(event.channel, format!("Available commands: {}", commands).as_slice());
    event.client.send_message(event.channel, format!("Use a command with `{}command_name` or `{}, command_name`", event.client.command_prefix, event.client.state.read().nick).as_slice());
}

pub fn register(register: &mut client::PluginRegister) {
    register.register_command("help", help);
}
