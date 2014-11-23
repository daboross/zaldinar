extern crate irc;

use irc::IrcClient;
use std::ascii::AsciiExt;

fn main() {
    let mut client = IrcClient::new("irc.spi.gt:6667");
    client.send("NICK bot");
    client.send("USER rust 0 * :Test");
    client.start_receiving();
    client.add_listener("ping", |client: &mut IrcClient, cmd: &str, args: &[&str], mask: Option<&str>| {
        client.send(format!("PONG {}", args[0]).as_slice());
    });
    client.add_listener("004", |client: &mut IrcClient, cmd: &str, args: &[&str], mask: Option<&str>| {
        client.send("JOIN #bot");
    });
    client.add_listener("privmsg", |client: &mut IrcClient, cmd: &str, args: &[&str], possible_mask: Option<&str>| {
        let mask = possible_mask.expect("PRIVMSG received without sender mask");
        if args[1].eq_ignore_ascii_case(":quit") && client.permitted.is_match(mask) {
            client.send("QUIT :Testing.");
        } else if args[1].eq_ignore_ascii_case(":raw") && client.permitted.is_match(mask) {
            client.send(args.slice_from(2).connect(" ").as_slice())
        } else {
            client.send(format!("PRIVMSG {}", args.connect(" ")).as_slice());
        }
    });
}
