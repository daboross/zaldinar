use std::io::prelude::*;
use std::ascii::AsciiExt;
use std::io;
use std::net;
use std::thread;
use std::sync;
use std::sync::mpsc;
use regex;
use fern;

use errors::InitializationError;
use client;

static IRC_COLOR_REGEX: regex::Regex = regex!("(\x03(\\d+,\\d+|\\d)|[\x0f\x02\x16\x1f\x02])");

pub struct IrcConnection {
    socket: net::TcpStream,
    data_out: Option<mpsc::Sender<IrcMessage>>,
    data_in: Option<mpsc::Receiver<Option<String>>>,
    client: sync::Arc<client::Client>,
}

impl IrcConnection {
    pub fn create(addr: &str, data_out: mpsc::Sender<IrcMessage>, data_in: mpsc::Receiver<Option<String>>,
            logger: fern::ArcLogger, client: sync::Arc<client::Client>)
            -> Result<(), io::Error> {
        let socket = try!(net::TcpStream::connect(addr));
        let connection_receiving = IrcConnection {
            socket: try!(socket.try_clone()),
            data_out: Some(data_out),
            data_in: None,
            client: client.clone(),
        };
        let connection_sending = IrcConnection {
            // No need to clone socket a second time, as this is the last time we are using it
            socket: socket,
            data_out: None,
            data_in: Some(data_in),
            client: client,
        };

        // Using unwrap() on these two because we know that data_out and data_in are Some
        connection_receiving.spawn_reading_thread(logger.clone()).unwrap();
        connection_sending.spawn_writing_thread(logger).unwrap();

        return Ok(());
    }

    fn spawn_reading_thread(self, logger: fern::ArcLogger) -> Result<(), InitializationError> {
        let data_out = match self.data_out {
            Some(ref v) => v.clone(),
            None => return Err(InitializationError::new(
                "Can't start reading thread without data_out")),
        };
        try!(thread::Builder::new().name("socket_reading_task".to_string()).spawn(move || {
            fern::local::set_thread_logger(logger);
            let mut reader = io::BufReader::new(self.socket);
            loop {
                let mut whole_input = String::new();
                if let Err(e) = reader.read_line(&mut whole_input) {
                    severe!("Error decoding IRC input: {}", e);
                    break;
                }
                if whole_input.is_empty() {
                    break; // end of file
                }
                let input = IRC_COLOR_REGEX.replace_all(whole_input.trim_right(), "");
                let message_split: Vec<&str> = input.split(' ').collect();
                let (command, args, possible_mask): (&str, &[&str], IrcMask) = if message_split[0]
                        .starts_with(":") {
                    (
                        message_split[1], &message_split[2..],
                        IrcMask::parse_from_str(&message_split[0][1..])
                    )
                } else {
                    (message_split[0], &message_split[1..], IrcMask::Nonexistent)
                };

                let ctcp = if command.eq_ignore_ascii_case("PRIVMSG")
                                && args[1].starts_with(":\x01")
                                && args[args.len() -1].ends_with("\x01") {
                    let ctcp_command;
                    let mut ctcp_message;
                    if args.len() > 2 {
                        ctcp_command = args[1][2..].to_string(); // to remove :\x01
                        ctcp_message = args[2..].connect(" ");
                        ctcp_message.pop(); // to remove last \x01
                    } else {
                        // remove starting :\x01 and ending \x01
                        ctcp_command = args[1][2..(args[1].len() - 1)].to_string();
                        ctcp_message = "".to_string();
                    }
                    Some((ctcp_command, ctcp_message))
                } else {
                    None
                };
                let channel = match command {
                    "353" => Some(args[2].to_string()),
                    "JOIN" | "PART" | "KICK" | "TOPIC" | "NOTICE" => Some(args[0].to_string()),
                    "PRIVMSG" => {
                        // This checks if the nick is the same as our bot's nick
                        // If the channel is our bots nick, and the sender has a nick, the message
                        // is a private message. For the sake of plugins trying to reply,  we set
                        // the channel to the sender's nick instead of our nick.
                        if args[0] == self.client.state.read().unwrap().nick {
                            match possible_mask.nick() {
                                Some(v) => Some(v.to_string()),
                                None => Some(args[0].to_string()),
                            }
                        } else {
                            Some(args[0].to_string())
                        }
                    },
                    _ => None,
                };
                // TODO: Change channel to sender nick if channel is our current nick.
                let args_owned: Vec<String> = args.iter().map(|s: &&str| s.to_string()).collect();
                let message = IrcMessage::new(command.to_string(), args_owned, possible_mask, ctcp,
                    channel);
                if let Err(_) = data_out.send(message) {
                    severe!("Failed to send to data_out from IrcConnection reading thread. \
                        Exiting.");
                    return;
                }
            }
        }));
        return Ok(());
    }

    fn spawn_writing_thread(mut self, logger: fern::ArcLogger) -> Result<(), InitializationError> {
        if (&self.data_in).is_none() {
            return Err(InitializationError::new("Can't start writing thread without data_in"));
        }
        try!(thread::Builder::new().name("socket_writing_task".to_string()).spawn(move || {
            fern::local::set_thread_logger(logger);
            let data_in = self.data_in.expect("Already confirmed above");
            loop {
                let command = match data_in.recv() {
                    Ok(Some(v)) => v,
                    Ok(None) | Err(_) => break,
                };
                if !command.starts_with("PONG ") {
                    info!(">>> {}", command);
                }
                log_error_then!(self.socket.write(command.as_bytes()), return,
                    "Failed to write to stream: {e}");
                log_error_then!(self.socket.write(&b"\n"), return,
                    "Failed to write to stream: {e}");
                log_error_then!(self.socket.flush(), return,
                    "Failed to write to stream: {e}");
            }
        }));
        return Ok(());
    }
}

pub enum IrcMask {
    Full(FullIrcMask),
    Unparseable(String),
    Nonexistent,
}

pub struct FullIrcMask {
    pub mask: String,
    pub nick: String,
    pub user: String,
    pub host: String,
}

pub struct IrcMessage {
    pub command: String,
    pub args: Vec<String>,
    pub mask: IrcMask,
    /// Option<(command, message)>
    pub ctcp: Option<(String, String)>,
    pub channel: Option<String>,
}

impl IrcMessage {
    fn new(command: String, args: Vec<String>, mask: IrcMask, ctcp: Option<(String, String)>,
            channel: Option<String>) -> IrcMessage {
        return IrcMessage {
            command: command,
            args: args,
            mask: mask,
            ctcp: ctcp,
            channel: channel,
        };
    }
}

impl IrcMask {
    fn new_full(mask: String, nick: String, user: String, host: String) -> IrcMask {
        return IrcMask::Full(FullIrcMask {
            mask: mask,
            nick: nick,
            user: user,
            host: host,
        });
    }

    fn new_mask_only(mask: String) -> IrcMask {
        return IrcMask::Unparseable(mask);
    }

    fn parse_from_str(mask: &str) -> IrcMask {
        let mask_split = mask.splitn(1, '!').collect::<Vec<&str>>();
        if mask_split.len() < 2 {
            return IrcMask::new_mask_only(mask.to_string());
        }
        let nick = mask_split[0];
        let user_and_host = mask_split[1];
        let user_and_host_split = user_and_host.splitn(1, '@').collect::<Vec<&str>>();
        if user_and_host_split.len() < 2 {
            return IrcMask::new_mask_only(mask.to_string());
        }
        let user = user_and_host_split[0];
        let host = user_and_host_split[1];
        return IrcMask::new_full(mask.to_string(), nick.to_string(), user.to_string(),
                                    host.to_string());
    }

    pub fn nick(&self) -> Option<&str> {
        match self {
            &IrcMask::Full(ref mask) => Some(&mask.nick),
            &IrcMask::Unparseable(_) => None,
            &IrcMask::Nonexistent => None
        }
    }
}
