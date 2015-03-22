use client::PluginRegister;
use events::CommandEvent;

fn help(event: &CommandEvent) {
    {
        let plugins = event.client.plugins.read().unwrap();
        let commands = plugins.commands.keys().collect::<Vec<&String>>().connect(", ");

        event.client.reply_notice(event, format!("Available commands: {}", commands));
        if event.client.is_mask_admin(event.mask()) {
            let plugins = event.client.plugins.read().unwrap();
            let admin_commands = plugins.admin_commands.keys()
                .collect::<Vec<&String>>().connect(", ");
            event.client.reply_notice(event, format!("Admin commands: {}", admin_commands));
        }
    }
    {
        let state = event.client.state.read().unwrap();
        event.client.send_message(event.channel(),
            format!("Use a command with `{}command_name` or `{}, command_name`",
                event.client.command_prefix, state.nick));
    }
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("help", help);
}
