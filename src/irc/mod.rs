extern crate regex;
#[phase(plugin)]
extern crate regex_macros;

use errors::InitializationError;
use std::io::{TcpStream, IoError, BufferedReader};

static IRC_COLOR_REGEX: regex::Regex = regex!("(\x03(\\d+,\\d+|\\d)|[\x0f\x02\x16\x1f])");

pub struct IrcConnection {
    socket: TcpStream,
    data_out: Option<Sender<IrcMessage>>,
    data_in: Option<Receiver<String>>
}

impl IrcConnection {
    pub fn create(addr: &str, data_out: Sender<IrcMessage>, data_in: Receiver<String>) -> Result<(), IoError> {
        let socket = try!(TcpStream::connect(addr));
        let connection_receiving = IrcConnection {
            socket: socket.clone(),
            data_out: Some(data_out),
            data_in: None
        };
        let connection_sending = IrcConnection {
            socket: socket, // No need to clone a second time, as this is the last time we are using this socket
            data_out: None,
            data_in: Some(data_in)
        };

        // Using unwrap() on these two because we know that data_out and data_in are Some() and not None
        connection_receiving.spawn_reading_thread().unwrap();
        connection_sending.spawn_writing_thread().unwrap();

        return Ok(())
    }

    fn spawn_reading_thread(self) -> Result<(), InitializationError> {
        let data_out = match self.data_out {
            Some(ref v) => v.clone(),
            None => return Err(InitializationError::new("Can't start reading thread without data_out"))
        };
        spawn(proc() {
            let mut reader = BufferedReader::new(self.socket.clone());
            loop {
                let whole_input = match reader.read_line() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Error in reading thread: {}", e);
                        break;
                    }
                };
                let input = IRC_COLOR_REGEX.replace_all(whole_input.trim_right(), "");
                let message_split: Vec<&str> = input.split(' ').collect();
                let (command, args, possible_mask): (&str, &[&str], Option<String>) = if message_split[0].starts_with(":") {
                    (message_split[1], message_split.slice_from(2), Some(message_split[0].slice_from(1).into_string()))
                } else {
                    (message_split[0], message_split.slice_from(1), None)
                };
                let args_owned: Vec<String> = args.iter().map(|s: &&str| s.to_string()).collect();
                let message = IrcMessage::new(command.into_string(), args_owned, possible_mask);
                data_out.send(message);
            }
        });
        return Ok(())
    }

    fn spawn_writing_thread(mut self) -> Result<(), InitializationError> {
        if (&self.data_in).is_none() {
            return Err(InitializationError::new("Can't start writing thread without data_in"))
        };
        spawn(proc() {
            let data_in = self.data_in.expect("Already confirmed above");
            loop {
                let command = data_in.recv();
                if !command.starts_with("PONG ") {
                    println!(">>> {}", command);
                }
                self.socket.write(command.as_bytes()).ok().expect("Failed to write to stream");
                self.socket.write(b"\n").ok().expect("Failed to write to stream");
                self.socket.flush().ok().expect("Failed to flush stream");
            }
        });
        return Ok(())
    }
}


pub struct IrcMessage {
    pub command: String,
    pub args: Vec<String>,
    pub mask: Option<String>
}

impl IrcMessage {
    fn new(command: String, args: Vec<String>, mask: Option<String>) -> IrcMessage {
        return IrcMessage {
            command: command,
            args: args,
            mask: mask
        }
    }
}
