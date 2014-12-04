/// This file contains commands to generally control and administer the bot.
use irc::Client;
use irc::interface::CommandEvent;

fn say(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args[0].starts_with("#") {
        event.client.send_message(event.args[0], event.args.slice_from(1).connect(" ").as_slice());
    } else {
        event.client.send_message(event.channel, event.args.connect(" ").as_slice());
    }
}

fn quit(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args.len() != 0 {
        event.client.quit(Some(event.args.connect(" ").as_slice()));
    } else {
        event.client.quit(None);
    }
}

fn raw(event: &CommandEvent) {
    event.client.send_raw(event.args.connect(" "));
    event.client.send_message(event.channel, "Sent raw message.");
}

fn join(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    event.client.join(event.args[0]);
    event.client.send_message(event.channel, format!("Joined {}.", event.args[0]).as_slice());
}

fn part(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    if event.args.len() > 1 {
        event.client.part(event.args[0], Some(event.args.slice_from(1).connect(" ").as_slice()));
    } else {
        event.client.part(event.args[0], None)
    }

    event.client.send_message(event.channel, format!("Parted {}.", event.args[0]).as_slice());
}

fn message(event: &CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    event.client.send_message(event.args[0], event.args.slice_from(1).connect(" ").as_slice());
    event.client.send_message(event.channel, format!("Sent message to {}.", event.args[0]).as_slice());
}

pub fn register(client: &mut Client) {
    client.add_command("say", say);
    client.add_command("message", message);
    client.add_command("raw", raw);
    client.add_command("join", join);
    client.add_command("part", part);
    client.add_command("leave", part);
    client.add_command("quit", quit);
}
