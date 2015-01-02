use client::PluginRegister;
use events::CommandEvent;

fn help(event: &CommandEvent) {
    let plugins = event.client.plugins.read().unwrap();
    let state = event.client.state.read().unwrap();
    let commands = plugins.commands.keys().collect::<Vec<&String>>().connect(", ");
    event.client.send_message(event.channel(),
        format!("Available commands: {}", commands).as_slice());
    event.client.send_message(event.channel(),
        format!("Use a command with `{}command_name` or `{}, command_name`",
            event.client.command_prefix, state.nick).as_slice());
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("help", help);
}
