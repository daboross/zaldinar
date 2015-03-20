extern crate "zaldinar-core" as zaldinar;

use zaldinar::client::PluginRegister;
use zaldinar::events::CommandEvent;
use zaldinar::client::ExecutingState;

fn act(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args[0].starts_with("#") {
        event.client.send_message(&*event.args[0], event.args[1..].connect(" "));
    } else {
        event.client.send_message(event.channel(), event.args.connect(" "));
    }
}

fn say(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args[0].starts_with("#") {
        event.client.send_message(&*event.args[0], event.args[1..].connect(" "));
    } else {
        event.client.send_message(event.channel(), event.args.connect(" "));
    }
}

fn quit(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args.len() != 0 {
        event.client.quit(Some(event.args.connect(" ")), ExecutingState::Done);
    } else {
        event.client.quit::<&str>(None, ExecutingState::Done);
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
    event.client.join(&*event.args[0]);
    event.client.send_message(event.channel(), format!("Joined {}.", event.args[0]));
}

fn part(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args.len() > 1 {
        event.client.part(&*event.args[0], Some(event.args[1..].connect(" ")));
    } else {
        event.client.part::<&str, &str>(&*event.args[0], None);
    }

    event.client.send_message(event.channel(), format!("Parted {}.", event.args[0]));
}

fn message(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    event.client.send_message(&*event.args[0], event.args[1..].connect(" "));
    event.client.send_message(event.channel(), format!("Sent message to {}.", event.args[0]));
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("say", say);
    register.register_command("message", message);
    // register.register_command("action", action);
    register.register_command("raw", raw);
    register.register_command("join", join);
    register.register_command("part", part);
    register.register_command("leave", part);
    register.register_command("quit", quit);
    // register.register_command("nick", nick);
}
