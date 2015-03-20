use client::PluginRegister;
use events::MessageEvent;

fn on_connect(event: &MessageEvent) {
    for command in &event.client.on_connect {
        event.client.send_command::<&str, &str>(command, &[]);
    }

    let nickserv = &event.client.nickserv;
    if nickserv.enabled {
        if nickserv.account.len() != 0 {
            event.client.send_message(&*nickserv.name, format!("{} {} {}",
                nickserv.command, nickserv.account, nickserv.password));
        } else {
            event.client.send_message(&*nickserv.name, format!("{} {}",
                nickserv.command, nickserv.password));
        }
    }

    for channel in &event.client.channels {
        event.client.send_command("JOIN", &[&**channel]);
    }
    {
        let mut state = event.client.state.write().unwrap();
        state.channels.extend(event.client.channels.iter().map(|s: &String| s.clone()));
    }
}

fn on_join(event: &MessageEvent) {
    if event.channel.is_none() {
        return;
    }
    if let Some(nick) = event.mask.nick() {
        let same = {
            let state = event.client.state.read().unwrap();
            nick == state.nick
        };
        if same {
            let mut state = event.client.state.write().unwrap();
            state.channels.push(event.channel().unwrap().to_string());
        }
    }
}

fn on_nick(event: &MessageEvent) {
    if let Some(nick) = event.mask.nick() {
        let same = {
            let state = event.client.state.read().unwrap();
            nick == state.nick
        };
        if same {
            let mut state = event.client.state.write().unwrap();
            state.nick = event.args[0].to_string();
        }
    }
}

pub fn register(register: &mut PluginRegister) {
    register.register_irc("004", on_connect);
    register.register_irc("join", on_join);
    register.register_irc("nick", on_nick);
}
