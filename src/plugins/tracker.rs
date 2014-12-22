use client::PluginRegister;
use interface::IrcMessageEvent;

fn on_connect(event: &IrcMessageEvent) {
    let nickserv = &event.client.nickserv;
    if nickserv.enabled {
        if nickserv.account.len() != 0 {
            event.client.send_message(nickserv.name.as_slice(), format!("{} {} {}", nickserv.command, nickserv.account, nickserv.password).as_slice());
        } else {
            event.client.send_message(nickserv.name.as_slice(), format!("{} {}", nickserv.command, nickserv.password).as_slice());
        }
    }

    for channel in event.client.channels.iter() {
        event.client.send_command("JOIN".into_string(), &[channel.as_slice()]);
    }
    event.client.state.write().channels.push_all(event.client.channels.iter().map(|s: &String| s.clone()).collect::<Vec<String>>().as_slice());
}

pub fn register(register: &mut PluginRegister) {
    register.register_irc("004", on_connect);
}
