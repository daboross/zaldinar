extern crate regex;
#[macro_use]
extern crate log;

macro_rules! regex {
    ($s:expr) => (::regex::Regex::new($s).unwrap())
}

use std::io::prelude::*;
use std::ascii::AsciiExt;
use std::io;
use std::net;
use std::thread;
use std::sync::mpsc;

/// This trait represents something which store an internal string. However, in order to allow for
/// the implementation to use an internal state like RwLock, this trait gives access using a
/// closure.
pub trait HasNick {
    fn with_current_nick<T, F>(&self, fun: F) -> T where F: Fn(&str) -> T;
}

pub fn connect<T>(addr: &str, data_out: mpsc::Sender<IrcMessage>,
        data_in: mpsc::Receiver<Option<String>>, client: T)
        -> io::Result<()> where T: HasNick + Send + 'static {
    let socket = try!(net::TcpStream::connect(addr));
    let irc_read = IrcRead {
        socket: io::BufReader::new(try!(socket.try_clone())),
        data_out: data_out,
        client: client,
    };
    let irc_write = IrcWrite {
        socket: socket,
        data_in: data_in,
    };

    try!(irc_read.spawn_reading_thread());
    try!(irc_write.spawn_writing_thread());

    return Ok(());
}

pub struct IrcRead<T: io::BufRead, C: HasNick> {
    socket: T,
    data_out: mpsc::Sender<IrcMessage>,
    client: C,
}

impl <T: io::BufRead + Send + 'static, C: HasNick + Send + 'static> IrcRead<T, C> {
    fn spawn_reading_thread(mut self) -> io::Result<thread::JoinHandle> {
        thread::Builder::new().name("irc_read_thread".to_string()).spawn(move || {
            self.read_loop();
        })
    }
}
impl <T: io::BufRead, C: HasNick> IrcRead<T, C> {
    /// Reads input from socket and sends parsed IrcMessages to data_out
    /// This will continue to read until either end of file is reached.
    /// or an error occurs either in `socket.read_line()` or `data_out.send()`.
    fn read_loop(&mut self) {
        // TODO: Move this back to a `const IRC_COLOR_REGEX` definition at the top of the file
        // once the regex_macros crate is available on stable rust
        let irc_color_regex = regex!("(\x03(\\d+,\\d+|\\d)|[\x0f\x02\x16\x1f\x02])");
        loop {
            let mut whole_input = String::new();
            if let Err(e) = self.socket.read_line(&mut whole_input) {
                error!("Error decoding IRC input: {}", e);
                break;
            }
            if whole_input.is_empty() {
                break; // end of file
            }
            let input = irc_color_regex.replace_all(whole_input.trim_right(), "");
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
                    if let Some(sender_nick) = possible_mask.nick() {
                        if self.client.with_current_nick(|nick| args[0] == nick) {
                            Some(sender_nick.to_string())
                        } else {
                            Some(args[0].to_string())
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
            if let Err(_) = self.data_out.send(message) {
                error!("Failed to send to data_out from IrcRead.");
                return;
            }
        }
    }
}

pub struct IrcWrite<T: io::Write> {
    socket: T,
    data_in: mpsc::Receiver<Option<String>>,
}

impl <T: io::Write + Send + 'static> IrcWrite<T> {
    fn spawn_writing_thread(mut self) -> io::Result<thread::JoinHandle> {
        thread::Builder::new().name("irc_write_thread".to_string()).spawn(move || {
            if let Err(e) = self.write_loop() {
                error!("Error writing to irc socket: {}", e);
            }
        })
    }
}

impl <T: io::Write> IrcWrite<T> {
    fn write_loop(&mut self) -> io::Result<()> {
        loop {
            let command = match self.data_in.recv() {
                Ok(Some(v)) => v,
                Ok(None) | Err(_) => break,
            };
            if !command.starts_with("PONG ") {
                info!(">>> {}", command);
            }
            try!(self.socket.write(command.as_bytes()));
            try!(self.socket.write(b"\n"));
            try!(self.socket.flush());
        }
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

    pub fn mask(&self) -> Option<&str> {
        match self {
            &IrcMask::Full(ref mask) => Some(&mask.mask),
            &IrcMask::Unparseable(ref mask) => Some(&mask),
            &IrcMask::Nonexistent => None
        }
    }
}
