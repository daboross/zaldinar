/// This file contains commands to generally control and administer the bot.
use client::{
    PluginRegister,
    ExecutingState,
};
use events::CommandEvent;

fn say(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args[0].starts_with("#") {
        event.client.send_message(event.args[0].as_slice(),
                                    event.args.slice_from(1).connect(" ").as_slice());
    } else {
        event.client.send_message(event.channel(), event.args.connect(" ").as_slice());
    }
}

fn quit(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args.len() != 0 {
        event.client.quit(Some(event.args.connect(" ").as_slice()), ExecutingState::Done);
    } else {
        event.client.quit(None, ExecutingState::Done);
    }
}

fn raw(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    event.client.send_raw(event.args.connect(" "));
    event.client.send_message(event.channel(), "Sent raw message.");
}

fn join(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    event.client.join(event.args[0].as_slice());
    event.client.send_message(event.channel(), format!("Joined {}.", event.args[0]).as_slice());
}

fn part(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args.len() > 1 {
        event.client.part(event.args[0].as_slice(),
                            Some(event.args.slice_from(1).connect(" ").as_slice()));
    } else {
        event.client.part(event.args[0].as_slice(), None);
    }

    event.client.send_message(event.channel(), format!("Parted {}.", event.args[0]).as_slice());
}

fn message(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    event.client.send_message(event.args[0].as_slice(),
                                event.args.slice_from(1).connect(" ").as_slice());
    event.client.send_message(event.channel(),
                                format!("Sent message to {}.", event.args[0]).as_slice());
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("say", say);
    register.register_command("message", message);
    register.register_command("raw", raw);
    register.register_command("join", join);
    register.register_command("part", part);
    register.register_command("leave", part);
    register.register_command("quit", quit);
}
