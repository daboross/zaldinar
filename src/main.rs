extern crate irc;

use irc::IrcClient;

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
            let command = if split[0].starts_with(":") {
                split[1]
            } else {
                split[0]
            };
            if command == "PING" {
                reader.send(format!("PONG {}", split[1]).as_slice());
            } else if command == "004" {
                reader.send("JOIN #dabo");
                reader.send("PRIVMSG #dabo :Hello!");
            } else if command == "PRIVMSG" {
                reader.send("QUIT :Testing.");
            }
        }
    });
}
