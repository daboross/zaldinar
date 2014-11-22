extern crate irc;

use irc::IrcClient;
use std::ascii::AsciiExt;

fn main() {
    let mut client = IrcClient::new("irc.spi.gt:6667");
    let mut reader = client.clone();

    client.send("NICK TestBot");
    client.send("USER rust 0 * :Test");

    spawn(proc() {
        reader.init_reader();
        loop {
            let input = match reader.read_line() {
                Ok(v) => v,
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            };
            println!("Received: {}", input.trim());
            let split: Vec<&str> = input.split(' ').collect();
            let (command, args) = if split[0].starts_with(":") {
                (split[1], split.slice_from(2))
            } else {
                (split[0], split.slice_from(1))
            };
            if command == "PING" {
                reader.send(format!("PONG {}", args[0]).as_slice());
            } else if command == "004" {
                reader.send("JOIN #testbot");
                reader.send("PRIVMSG #testbot :Hello!");
            } else if command == "PRIVMSG" {
                if args[args.len() - 1].trim().eq_ignore_ascii_case(":quit") {
                    reader.send("QUIT :Testing.");
                    break;
                } else {
                    println!("{}", args[args.len() - 1]);
                    reader.send(format!("PRIVMSG {}", args.connect(" ")).as_slice());
                }
            }
        }
    });
}
