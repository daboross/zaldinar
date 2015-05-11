use client::PluginRegister;
use events::CommandEvent;

fn help(event: &CommandEvent) {
    {
        let plugins = event.client.plugins().read().unwrap();
        // TODO: `.map(|s| &**s)` is a workaround for changes made in
        // https://github.com/rust-lang/rust/pull/25162 - it should be removed when (if) it
        // becomes no longer required
        let commands = plugins.commands.keys().map(|s| &**s).collect::<Vec<_>>().connect(", ");

        event.client.reply_notice(event, format!("Available commands: {}", commands));
        if event.client.is_mask_admin(event.mask()) {
            let plugins = event.client.plugins().read().unwrap();
            // TODO: `.map(|s| &**s)` is a workaround for changes made in
            // https://github.com/rust-lang/rust/pull/25162 - it should be removed when (if) it
            // becomes no longer required
            let admin_commands = plugins.admin_commands.keys().map(|s| &**s)
                .collect::<Vec<_>>().connect(", ");

            event.client.reply_notice(event, format!("Admin commands: {}", admin_commands));
        }
    }
    {
        let state = event.client.state().read().unwrap();
        event.client.reply_notice(event,
            format!("Use a command with `{}command_name` or `{}, command_name`",
                event.client.command_prefix, state.nick));
    }
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("help", help);
}
