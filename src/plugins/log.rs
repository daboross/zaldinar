use std::ascii::AsciiExt;

use client::PluginRegister;
use interface::IrcMessageEvent;

fn log_message(event: &IrcMessageEvent) {
    let nick = event.mask.nick().unwrap_or_else(|| event.mask.mask().unwrap_or("*unknown*"));
    let message = match event.command.to_ascii_upper().as_slice() {
        "PRIVMSG" => match event.ctcp {
            Some((ctcp_command, ctcp_message)) => match ctcp_command {
                "ACTION" => format!("[{}] * {} {}", event.args[0], nick, ctcp_message),
                _ => if ctcp_message.len() == 0 {
                    format!("[{}] CTCP {} from {}", event.args[0], ctcp_command, nick)
                } else {
                    format!("[{}] CTCP {} from {}: {}", event.args[0], ctcp_command, nick, ctcp_message)
                },
            },
            None => format!("[{}] <{}> {}", event.args[0], nick, event.args.slice_from(1).connect(" ").slice_from(1)),
        },
        "NOTICE" => format!("[{}] -{}- {}", event.args[0], nick, event.args.slice_from(1).connect(" ").slice_from(1)),
        "JOIN" => format!("[{}] *** {} joined", event.args[0], nick),
        "PART" => format!("[{}] *** {} left ({})", event.args[0], nick, event.args[1]),
        "KICK" => {
            if event.args.len() > 2 {
                format!("[{}] *** {} kicked {} ({})", event.args[0], nick, event.args[1], event.args.slice_from(2).connect(" ").slice_from(1))
            } else {
                format!("[{}] *** {} kicked {}", event.args[0], nick, event.args[1])
            }
        },
        "TOPIC" => format!("[{}] *** {} changed the topic to \"{}\"", event.args[0], nick, event.args.slice_from(1).connect(" ").slice_from(1)),
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
