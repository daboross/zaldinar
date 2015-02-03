use std::ascii::AsciiExt;

use client::PluginRegister;
use events::MessageEvent;

fn log_message(event: &MessageEvent) {
    let nick = event.mask.nick().unwrap_or_else(|| event.mask.mask().unwrap_or("*unknown*"));
    let message = match &event.command.to_ascii_uppercase()[] {
        "PRIVMSG" => match event.ctcp() {
            Some((ctcp_command, ctcp_message)) => match ctcp_command {
                "ACTION" => format!("[{}] * {} {}", event.args[0], nick, ctcp_message),
                _ => if ctcp_message.len() == 0 {
                    format!("[{}] CTCP {} from {}", event.args[0], ctcp_command, nick)
                } else {
                    format!("[{}] CTCP {} from {}: {}", event.args[0], ctcp_command, nick,
                        ctcp_message)
                },
            },
            None => format!("[{}] <{}> {}", event.args[0], nick, &event.args[1..].connect(" ")[1..]),
        },
        "NOTICE" => format!("[{}] -{}- {}", event.args[0], nick, &event.args[1..].connect(" ")[1..]),
        "JOIN" => format!("[{}] *** {} joined", event.args[0], nick),
        "PART" => {
            if event.args.len() > 1 {
                format!("[{}] *** {} left ({})", event.args[0], nick, event.args[1])
            } else {
                format!("[{}] *** {} left", event.args[0], nick)
            }

        },
        "KICK" => {
            if event.args.len() > 2 {
                format!("[{}] *** {} kicked {} ({})", event.args[0], nick, event.args[1],
                    &event.args[2..].connect(" ")[1..])
            } else {
                format!("[{}] *** {} kicked {}", event.args[0], nick, event.args[1])
            }
        },
        "TOPIC" => format!("[{}] *** {} changed the topic to \"{}\"", event.args[0], nick,
            &event.args[1..].connect(" ")[1..]),
        "PING" => return, // don't log pings
        _ => match event.mask.mask() {
            Some(mask) => format!("{} {} {}", mask, event.command, event.args.connect(" ")),
            None => format!("{} {}", event.command, event.args.connect(" ")),
        }
    };
    info!("{}", message);
}

pub fn register(register: &mut PluginRegister) {
    register.register_catch_all(log_message);
}
