use client::PluginRegister;
use interface::IrcMessageEvent;

fn on_connect(event: &IrcMessageEvent) {
    for command in event.client.on_connect.iter() {
        event.client.send_command(command.clone(), &[]);
    }

    let nickserv = &event.client.nickserv;
    if nickserv.enabled {
        if nickserv.account.len() != 0 {
            event.client.send_message(nickserv.name.as_slice(), format!("{} {} {}", nickserv.command, nickserv.account, nickserv.password).as_slice());
        } else {
            event.client.send_message(nickserv.name.as_slice(), format!("{} {}", nickserv.command, nickserv.password).as_slice());
        }
    }

    for channel in event.client.channels.iter() {
        event.client.send_command("JOIN".to_string(), &[channel.as_slice()]);
    }
    event.client.state.write().channels.push_all(event.client.channels.iter().map(|s: &String| s.clone()).collect::<Vec<String>>().as_slice());
}

fn on_join(event: &IrcMessageEvent) {
    if event.channel.is_none() {
        return;
    }
    match event.mask.nick() {
        Some(nick) => if nick == event.client.state.read().nick {
            event.client.state.write().channels.push(event.channel.unwrap().to_string());
        },
        None => (),
    }
}

fn on_nick(event: &IrcMessageEvent) {
    match event.mask.nick() {
        Some(nick) => if nick == event.client.state.read().nick {
            event.client.state.write().nick = event.args[0].to_string();
        },
        None => (),
    }
}

pub fn register(register: &mut PluginRegister) {
    register.register_irc("004", on_connect);
    register.register_irc("join", on_join);
    register.register_irc("nick", on_nick);
}
