/// This file contains commands to generally control and administer the bot.
use irc::Client;
use irc::interface::CommandEvent;

fn say(event: &mut CommandEvent) {
    if event.args[0].starts_with("#") {
        if !event.client.is_admin(event) {
            return;
        }
        event.client.send_message(event.args[0], &*event.args.slice_from(1).connect(" "));
    } else {
        event.client.send_message(event.channel, &*event.args.connect(" "));
    }
}

fn quit(event: &mut CommandEvent) {
    if !event.client.is_admin(event) {
        return
    }
    event.client.send_command("QUIT".to_string(), &[]);
}

fn raw(event: &mut CommandEvent) {
    event.client.send_raw(event.args.connect(" "));
    event.client.send_message(event.channel, "Sent raw message.");
}

fn join(event: &mut CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    event.client.join(event.args[0]);
    event.client.send_message(event.channel, "Joined channel.");
}

fn message(event: &mut CommandEvent) {
    if !event.client.is_admin(event) {
        return;
    }
    event.client.send_message(event.args[0], event.args.slice_from(1).connect(" ").as_slice());
    event.client.send_message(event.channel, format!("Sent message to {}.", event.args[0]).as_slice());
}

pub fn register(client: &mut Client) {
    client.add_command("say", say);
    client.add_command("quit", quit);
    client.add_command("raw", raw);
    client.add_command("join", join);
    client.add_command("message", message);
}
