
#![feature(phase)]

#[phase(plugin)]
extern crate regex_macros;

extern crate regex;
extern crate irc;

// TODO: Add this to the config
static PERMITTED: regex::Regex = regex!(r"^Dabo[^!]*![^@]*@me.dabo.guru$");

fn main() {
    let config = match irc::load_config_from_file(&Path::new("config.json")) {
        Ok(v) => v,
        Err(e) => {
            println!("Error loading configuration: {}", e);
            std::os::set_exit_status(1);
            return
        }
    };

    let mut client = irc::Client::new(config);

    client.add_listener("004", |event: &mut irc::IrcMessageEvent| {
        for channel in event.client.config.channels.iter() {
            event.client.send_command("JOIN".into_string(), &[channel.as_slice()]);
        }
    });

    client.add_command("say", |event: &mut irc::CommandEvent| {
        event.client.send_raw(format!("PRIVMSG {} :{}", event.channel, event.args.connect(" ")));
    });

    client.add_command("quit", |event: &mut irc::CommandEvent| {
        if event.mask.is_none() || !PERMITTED.is_match(event.mask.unwrap()) {
            event.client.send_command("PRIVMSG".to_string(), &[event.channel, ":Permission denied."]);
            return;
        }
        event.client.send_command("QUIT".to_string(), &[":See you!"]);
    });

    client.add_command("raw", |event: &mut irc::CommandEvent| {
        if event.mask.is_none() || !PERMITTED.is_match(event.mask.unwrap()) {
            event.client.send_command("PRIVMSG".to_string(), &[event.channel, ":Permission denied."]);
            return;
        }
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
