extern crate regex;
#[phase(plugin)]
extern crate regex_macros;

use std::ascii::AsciiExt;
use std::io;
use std::io::{TcpStream, IoError, BufferedReader};
use std::task::TaskBuilder;

use errors::InitializationError;

static IRC_COLOR_REGEX: regex::Regex = regex!("(\x03(\\d+,\\d+|\\d)|[\x0f\x02\x16\x1f\x02])");

pub struct IrcConnection {
    socket: TcpStream,
    data_out: Option<Sender<Option<IrcMessage>>>,
    data_in: Option<Receiver<Option<String>>>,
}

impl IrcConnection {
    pub fn create(addr: &str, data_out: Sender<Option<IrcMessage>>, data_in: Receiver<Option<String>>) -> Result<(), IoError> {
        let socket = try!(TcpStream::connect(addr));
        let connection_receiving = IrcConnection {
            socket: socket.clone(),
            data_out: Some(data_out),
            data_in: None,
        };
        let connection_sending = IrcConnection {
            socket: socket, // No need to clone a second time, as this is the last time we are using this socket
            data_out: None,
            data_in: Some(data_in),
        };

        // Using unwrap() on these two because we know that data_out and data_in are Some() and not None
        connection_receiving.spawn_reading_thread().unwrap();
        connection_sending.spawn_writing_thread().unwrap();

        return Ok(());
    }

    fn spawn_reading_thread(self) -> Result<(), InitializationError> {
        let data_out = match self.data_out {
            Some(ref v) => v.clone(),
            None => return Err(InitializationError::new("Can't start reading thread without data_out")),
        };
        TaskBuilder::new().named("socket_reading_task").spawn(move || {
            let mut reader = BufferedReader::new(self.socket.clone());
            loop {
                let whole_input = match reader.read_line() {
                    Ok(v) => v,
                    Err(e) => {
                        match e.kind {
                            io::IoErrorKind::EndOfFile => (),
                            _ => println!("Error in reading thread: {}", e),
                        }
                        data_out.send(None);
                        break;
                    },
                };
                let input = IRC_COLOR_REGEX.replace_all(whole_input.trim_right(), "");
                let message_split: Vec<&str> = input.split(' ').collect();
                let (command, args, possible_mask): (&str, &[&str], IrcMask) = if message_split[0].starts_with(":") {
                    (message_split[1], message_split.slice_from(2), IrcMask::parse_from_str(message_split[0].slice_from(1)))
                } else {
                    (message_split[0], message_split.slice_from(1), IrcMask::Nonexistent)
                };
                let ctcp = if command.eq_ignore_ascii_case("PRIVMSG")
                              && args[1].starts_with(":\x01")
                              && args[args.len() -1].ends_with("\x01") {
                    let ctcp_command;
                    let mut ctcp_message;
                    if args.len() > 2 {
                        ctcp_command = args[1].slice_from(2).into_string(); // to remove :\x01
                        ctcp_message = args.slice_from(2).connect(" ");
                        ctcp_message.pop(); // to remove last \x01
                    } else {
                        ctcp_command = args[1].slice(2, args[1].len() - 1).into_string(); // remove starting :\x01 and ending \x01
                        ctcp_message = "".into_string();
                    }
                    Some((ctcp_command, ctcp_message))
                } else {
                    None
                };
                let args_owned: Vec<String> = args.iter().map(|s: &&str| s.into_string()).collect();
                let message = IrcMessage::new(command.into_string(), args_owned, possible_mask, ctcp);
                data_out.send(Some(message));
            }
        });
        return Ok(());
    }

    fn spawn_writing_thread(mut self) -> Result<(), InitializationError> {
        if (&self.data_in).is_none() {
            return Err(InitializationError::new("Can't start writing thread without data_in"));
        }
        TaskBuilder::new().named("socket_writing_task").spawn(move || {
            let data_in = self.data_in.expect("Already confirmed above");
            loop {
                let command = match data_in.recv() {
                    Some(v) => v,
                    None => break,
                };
                if !command.starts_with("PONG ") {
                    println!(">>> {}", command);
                }
                self.socket.write(command.as_bytes()).ok().expect("Failed to write to stream");
                self.socket.write(b"\n").ok().expect("Failed to write to stream");
                self.socket.flush().ok().expect("Failed to flush stream");
            }
        });
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
}

impl IrcMessage {
    fn new(command: String, args: Vec<String>, mask: IrcMask, ctcp: Option<(String, String)>) -> IrcMessage {
        return IrcMessage {
            command: command,
            args: args,
            mask: mask,
            ctcp: ctcp,
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
            return IrcMask::new_mask_only(mask.into_string());
        }
        let nick = mask_split[0];
        let user_and_host = mask_split[1];
        let user_and_host_split = user_and_host.splitn(1, '@').collect::<Vec<&str>>();
        if user_and_host_split.len() < 2 {
            return IrcMask::new_mask_only(mask.into_string());
        }
        let user = user_and_host_split[0];
        let host = user_and_host_split[1];
        return IrcMask::new_full(mask.into_string(), nick.into_string(), user.into_string(), host.into_string());
    }
}
