extern crate "zaldinar-core" as zaldinar;

use zaldinar::client::PluginRegister;
use zaldinar::events::CommandEvent;
use zaldinar::client::ExecutingState;

fn action(event: &CommandEvent) {
    if event.args[0].starts_with("#") {
        event.client.send_ctcp(&*event.args[0], "ACTION", event.args[1..].connect(" "));
    } else {
        event.client.send_ctcp(event.channel(), "ACTION", event.args.connect(" "));
    }
}

fn say(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    let (channel, message) = if event.args[0].starts_with("#") {
        (&*event.args[0], event.args[1..].connect(" "))
    } else {
        (event.channel(), event.args.connect(" "))
    };
    event.client.send_message(channel, message);
    event.client.reply_notice(event, format!("Sent message to {}.", channel));
}

fn quit(event: &CommandEvent) {
    if event.args.len() != 0 {
        event.client.quit(Some(event.args.connect(" ")), ExecutingState::Done);
    } else {
        event.client.quit::<&str>(None, ExecutingState::Done);
    }
}

fn raw(event: &CommandEvent) {
    event.client.send_raw(event.args.connect(" "));
    event.client.reply_notice(event, "Sent raw message.");
}

fn join(event: &CommandEvent) {
    if event.args.is_empty() {
        event.client.reply_notice(event, "Please specify a channel to join.");
        return;
    }
    event.client.join(&*event.args[0]);
    event.client.reply_notice(event, format!("Joined {}.", event.args[0]));
}

fn part(event: &CommandEvent) {
    let (channel, reason) = if event.args.is_empty() {
        (event.channel(), None)
    } else if event.args[0].starts_with("#") {
        if event.args.len() > 1 {
            (&*event.args[0], Some(event.args[1..].connect(" ")))
        } else {
            (&*event.args[0], None)
        }
    } else {
        (event.channel(), Some(event.args.connect(" ")))
    };
    event.client.part(channel, reason);
    event.client.reply_notice(event, format!("Parted {}.", channel));
}

fn message(event: &CommandEvent) {
    if event.args.len() < 2 {
        event.client.reply_notice(event, "Please specify both a channel and a message to send");
        return;
    }
    event.client.send_message(&*event.args[0], event.args[1..].connect(" "));
    event.client.reply_notice(event, format!("Sent message to {}.", event.args[0]));
}

pub fn register(register: &mut PluginRegister) {
    register.register_admin_command("say", say);
    register.register_admin_command("message", message);
    register.register_admin_command("action", action);
    register.register_admin_command("raw", raw);
    register.register_admin_command("join", join);
    register.register_admin_command("part", part);
    register.register_admin_command("quit", quit);
    // register.register_command("nick", nick);
}
