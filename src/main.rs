
#![feature(phase)]

#[phase(plugin)]
extern crate regex_macros;

extern crate regex;
extern crate irc;

// TODO: Add this to the config
static PERMITTED: regex::Regex = regex!(r"^Dabo[^!]*![^@]*@me.dabo.guru$");

fn main() {
    let mut client = irc::Client::new(irc::load_config_from_file(&Path::new("config.json")).unwrap());

    client.add_listener("004", |event: &mut irc::IrcMessageEvent| {
        // TODO: Give access to config data via IrcInterface so that we can join configured channels
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

    // TODO: Add this to Client
    client.interface.send_command("NICK".into_string(), &[&*client.interface.config.nick]);
    client.interface.send_command("USER".into_string(), &[&*client.interface.config.user, "0", "*", &*format!(":{}", client.interface.config.real_name)]);

    client.connect().ok().expect("Failed to connect!");
}
