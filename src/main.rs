#![feature(phase)]
#[phase(plugin)]

extern crate regex_macros;
extern crate regex;
extern crate irc;

use irc::{IrcClient, IrcEvent};
use std::ascii::AsciiExt;

fn main() {
    let mut client = IrcClient::new("irc.spi.gt:6667");
    client.send("NICK bot");
    client.send("USER rust 0 * :Test");
    client.start_receiving();
    client.add_listener("ping", |event: &mut IrcEvent| {
        event.client.send(format!("PONG {}", event.args[0]).as_slice());
    });
    client.add_listener("004", |event: &mut IrcEvent| {
        event.client.send("JOIN #bot");
    });
    client.add_listener("privmsg", |event: &mut IrcEvent| {
        let permitted = regex!(r"^Dabo[^!]*![^@]*@me.dabo.guru$");
        let mask = event.mask.expect("PRIVMSG received without sender mask");
        if event.args[1].eq_ignore_ascii_case(":quit") && permitted.is_match(mask) {
            event.client.send("QUIT :Testing.");
        } else if event.args[1].eq_ignore_ascii_case(":raw") && permitted.is_match(mask) {
            event.client.send(event.args.slice_from(2).connect(" ").as_slice())
        } else {
            event.client.send(format!("PRIVMSG {}", event.args.connect(" ")).as_slice());
        }
    });
}
