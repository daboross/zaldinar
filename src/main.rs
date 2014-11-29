
#![feature(phase)]

#[phase(plugin)]
extern crate regex_macros;

extern crate regex;
extern crate irc;

fn main() {
    let config = match irc::load_config_from_file(&Path::new("config.json")) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            std::os::set_exit_status(1);
            return
        }
    };

    let mut client = match irc::Client::new(config) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            std::os::set_exit_status(1);
            return
        }
    };

    client.add_listener("004", |event: &mut irc::IrcMessageEvent| {
        for channel in event.client.config.channels.iter() {
            event.client.send_command("JOIN".into_string(), &[channel.as_slice()]);
        }
    });

    client.add_command("say", |event: &mut irc::CommandEvent| {
        if event.args[0].starts_with("#") {
            if !event.client.is_admin(event) {
                return;
            }
            event.client.send_message(event.args[0], &*event.args.slice_from(1).connect(" "));
        } else {
            event.client.send_message(event.channel, &*event.args.connect(" "));
        }
    });

    client.add_command("quit", |event: &mut irc::CommandEvent| {
        if !event.client.is_admin(event) {
            return
        }
        event.client.send_command("QUIT".to_string(), &[":See you!"]);
    });

    client.add_command("raw", |event: &mut irc::CommandEvent| {
        event.client.send_raw(event.args.connect(" "));
    });

    match client.connect() {
        Ok(()) => (),
        Err(e) => {
            println!("Error connecting: {}", e);
            std::os::set_exit_status(1);
            // There is no need to stop other tasks at this point, because the only time client.connect() returns Err is before any tasks are started
            return
        }
    }
}
