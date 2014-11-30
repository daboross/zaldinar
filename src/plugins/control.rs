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
    event.client.send_command("QUIT".to_string(), &[":See you!"]);
}

fn raw(event: &mut CommandEvent) {
    event.client.send_raw(event.args.connect(" "));
}

fn join(event &mut CommandEvent) {
    event.client.send_command("JOIN".to_string(), &[":"])
}

pub fn register(client: &mut Client) {
    client.add_command("say", say);
    client.add_command("quit", quit);
    client.add_command("raw", raw);
}
