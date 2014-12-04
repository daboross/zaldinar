/// This file contains commands to generally control and administer the bot.
use irc::Client;
use irc::interface::IrcMessageEvent;
use std::ascii::AsciiExt;

fn log_message(event: &IrcMessageEvent) {
    let mask = event.mask.unwrap_or("*unknown*");
    let message = match event.command.to_ascii_upper().as_slice() {
        "PRIVMSG" => match event.ctcp {
            Some((ctcp_command, ctcp_message)) => match ctcp_command {
                "ACTION" => format!("[{}] * {} {}", event.args[0], mask, ctcp_message),
                _ => format!("[{}] <{}> CTCP {} {}", event.args[0], mask, ctcp_command, ctcp_message),
            },
            None => format!("[{}] <{}> {}", event.args[0], mask, event.args.slice_from(1).connect(" ").slice_from(1)),
        },
        "NOTICE" => format!("[{}] -{}- {}", event.args[0], mask, event.args.slice_from(1).connect(" ").slice_from(1)),
        "JOIN" => format!("[{}] *** {} joined", event.args[0], mask),
        "PART" => format!("[{}] *** {} left ({})", event.args[0], event.mask, event.args[1]),
        "KICK" => {
            if event.args.len() > 2 {
                format!("[{}] *** {} kicked {} ({})", event.args[0], mask, event.args[1], event.args.slice_from(2).connect(" ").slice_from(1))
            } else {
                format!("[{}] *** {} kicked {}", event.args[0], mask, event.args[1])
            }
        },
        "TOPIC" => format!("[{}] *** {} changed the topic to \"{}\"", event.args[0], event.mask.unwrap(), event.args.slice_from(1).connect(" ").slice_from(1)),
        "PING" => return, // don't log pings
        _ => match event.mask {
            Some(mask) => format!("{} {} {}", mask, event.command, event.args.connect(" ")),
            None => format!("{} {}", event.command, event.args.connect(" ")),
        }
    };
    println!("{}", message);
}

pub fn register(client: &mut Client) {
    client.add_catch_all_listener(log_message);
}
