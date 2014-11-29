#![feature(globs)]

extern crate serialize;
extern crate regex;

use std::io::*;
use std::sync::{Arc, RWLock};
use std::ascii::AsciiExt;
use std::collections::HashMap;
use serialize::json;
use regex::Regex;
use std::error::Error;
use serialize::json::DecoderError;
use serialize::json::ParserError;

pub enum InitializationError {
    Io(IoError),
    Regex(regex::Error),
    Decoder(DecoderError),
    Other(String)
}

impl InitializationError {
    fn new(detail: &str) -> InitializationError {
        InitializationError::Other(detail.to_string())
    }

    fn from_string(detail: String) -> InitializationError {
        InitializationError::Other(detail)
    }
}

impl std::error::FromError<IoError> for InitializationError {
    fn from_error(error: IoError) -> InitializationError {
        InitializationError::Io(error)
    }
}

impl std::error::FromError<regex::Error> for InitializationError {
    fn from_error(error: regex::Error) -> InitializationError {
        InitializationError::Regex(error)
    }
}

impl std::error::FromError<DecoderError> for InitializationError {
    fn from_error(error: DecoderError) -> InitializationError {
        InitializationError::Decoder(error)
    }
}

impl std::fmt::Show for InitializationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            InitializationError::Io(ref v) => v.fmt(formatter),
            InitializationError::Regex(ref v) => v.fmt(formatter),
            InitializationError::Decoder(ref v) => v.fmt(formatter),
            InitializationError::Other(ref v) => v.fmt(formatter)
        }
    }
}

struct IrcConnection {
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
                        println!("Error: {}", e);
                        break;
                    }
                };
                let input = whole_input.trim_right();
                let message_split: Vec<&str> = input.split(' ').collect();
                let (command, args, possible_mask): (&str, &[&str], Option<String>) = if message_split[0].starts_with(":") {
                    (message_split[1], message_split.slice_from(2), Some(message_split[0].slice_from(1).into_string()))
                } else {
                    (message_split[0], message_split.slice_from(1), None)
                };
                match possible_mask {
                    Some(ref v) => println!("Received {} from {}: {}", command, v, args.connect(" ")),
                    None => println!("Received {}: {}", command, args.connect(" "))
                }
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
                println!("Sending: {}", command);
                self.socket.write(command.as_bytes()).ok().expect("Failed to write to stream");
                self.socket.write(b"\n").ok().expect("Failed to write to stream");
                self.socket.flush().ok().expect("Failed to flush stream");
            }
        });
        return Ok(())
    }
}


struct IrcMessage {
    command: String,
    args: Vec<String>,
    mask: Option<String>
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

// TODO: Store channels joined
pub struct Client {
    data_in: Receiver<IrcMessage>,
    pub interface: IrcInterface,
    commands: Arc<RWLock<HashMap<String, Vec<|&mut CommandEvent|:'static>>>>,
    raw_listeners: Arc<RWLock<HashMap<String, Vec<|&mut IrcMessageEvent|:'static>>>>,
    config: Arc<ClientConfiguration>,
    irc_connection_channel: Option<(Sender<IrcMessage>, Receiver<String>)>,
}

impl Client {
    pub fn new(config: ClientConfiguration) -> Result<Client, InitializationError> {
        let rc_config = Arc::new(config);
        let (data_out, connection_data_in) = channel();
        let (connection_data_out, data_in) = channel();
        let client = Client {
            interface: try!(IrcInterface::new(data_out, rc_config.clone())),
            data_in: data_in,
            commands: Arc::new(RWLock::new(HashMap::new())),
            raw_listeners: Arc::new(RWLock::new(HashMap::new())),
            config: rc_config,
            irc_connection_channel: Some((connection_data_out, connection_data_in))
        };
        return Ok(client);
    }

    pub fn connect(mut self) -> Result<(), InitializationError> {
        // Get connection data_out/data_in, and assure that we haven't already done this (ensure we aren't already connected)
        let (connection_data_out, connection_data_in) = match self.irc_connection_channel {
            Some(v) => v,
            None => return Err(InitializationError::new("Already connected"))
        };
        self.irc_connection_channel = None;

        // Send NICK and USER, the initial IRC commands. Because an IrcConnection hasn't been created to receive these yet,
        //  they will just go on hold and get sent as soon as the IrcConnection connects.
        self.interface.send_command("NICK".into_string(), &[&*self.config.nick]);
        self.interface.send_command("USER".into_string(), &[&*self.config.user, "0", "*", &*format!(":{}", self.config.real_name)]);

        try!(IrcConnection::create(self.config.address.as_slice(), connection_data_out, connection_data_in));
        self.spawn_dispatch_thread();
        return Ok(())
    }

    pub fn add_listener(&mut self, irc_command: &str, f: |&mut IrcMessageEvent|:'static) {
        let command_string = irc_command.into_string().to_ascii_lower();
        let mut listener_map = self.raw_listeners.write();
        {
            // I can't use a match here because then I would be borrowing listener_map mutably twice:
            // Once for the match statement, and a second time inside the None branch
            if listener_map.contains_key(&command_string) {
                listener_map.get_mut(&command_string).expect("Honestly, this won't happen.").push(f);
            } else {
                listener_map.insert(command_string, vec!(f));
            }
        }
        listener_map.downgrade();
    }

    pub fn add_command(&mut self, command: &str, f: |&mut CommandEvent|:'static) {
        let command_lower = command.into_string().to_ascii_lower();
        let mut command_map = self.commands.write();
        {
            // I can't use a match here because then I would be borrowing the command_map mutably twice:
            // Once for the match statement, and a second time inside the None branch
            if command_map.contains_key(&command_lower) {
                command_map.get_mut(&command_lower).expect("Honestly, this won't happen.").push(f);
            } else {
                command_map.insert(command_lower, vec!(f));
            }
        }
        command_map.downgrade();
    }

    fn spawn_dispatch_thread(self) {
        spawn(proc() {
            loop {
                let message: IrcMessage = self.data_in.recv();
                self.process_message(&message);
            }
        });
    }

    // Noting: This has to be a separate method from spawn_dispatch_thread, so that we can name an 'a lifetime.
    // This allows us to give the new &str slices a specific lifetime, which I don't know a way to do without making a new function.
    fn process_message<'a>(&self, message: &'a IrcMessage) {
        let shared_mask: Option<&str> = message.mask.as_ref().map(|s| &**s);
        let shared_args = message.args.iter().map(|s| &**s).collect::<Vec<&'a str>>();

        // PING
        if message.command.as_slice().eq_ignore_ascii_case("PING") {
            self.interface.send_command("PONG".into_string(), shared_args.as_slice());
        }

        // Raw listeners
        let message_event = &mut IrcMessageEvent::new(&self.interface, message.command.as_slice(), shared_args.as_slice(), shared_mask);
        let mut listener_map = self.raw_listeners.write();
        // New scope so that listeners will go out of scope before we run listener_map.downgrade()
        {
            let listeners = listener_map.get_mut(&message.command.to_ascii_lower());
            if listeners.is_some() {
                for listener in listeners.unwrap().iter_mut() {
                    (*listener)(message_event);
                }
            }
        }
        listener_map.downgrade();

        // Commands
        if message.command.as_slice().eq_ignore_ascii_case("PRIVMSG") {
            let channel = shared_args[0];
            let prefix = format!(":{}", self.config.command_prefix.as_slice());
            if shared_args[1].starts_with(prefix.as_slice()) {
                let command = shared_args[1].slice_from(prefix.len()).into_string().to_ascii_lower();
                let mut command_map = self.commands.write();
                {
                    let commands = command_map.get_mut(&command);
                    if commands.is_some() {
                        let args = shared_args.slice_from(2);
                        let command_event = &mut CommandEvent::new(&self.interface, channel, args, shared_mask);

                        for command in commands.unwrap().iter_mut() {
                            (*command)(command_event);
                        }
                    }
                }
                command_map.downgrade();
            }
        }
    }
}

pub struct IrcInterface {
    data_out: Sender<String>,
    pub config: Arc<ClientConfiguration>,
    admins: Arc<Vec<Regex>>
}

impl IrcInterface {
    fn new(data_out: Sender<String>, config: Arc<ClientConfiguration>) -> Result<IrcInterface, InitializationError> {
        let mut admins: Vec<Regex> = Vec::new();
        for admin_str in config.admins.iter() {
            admins.push(try!(Regex::new(format!("^{}$", admin_str.as_slice()).as_slice())));
        }
        let interface = IrcInterface {
            data_out: data_out,
            config: config,
            admins: Arc::new(admins)
        };
        return Ok(interface)
    }
    pub fn send_raw(&self, line: String) {
        self.data_out.send(line);
    }

    pub fn send_command<'a>(&self, command: String, args: &[&str]) {
        let mut line = command;
        line.push(' ');
        line.push_str(args.connect(" ").as_slice());
        self.send_raw(line);
    }

    pub fn send_message(&self, target: &str, message: &str) {
        let line = format!("PRIVMSG {} :{}", target, message);
        self.send_raw(line);
    }

    pub fn is_admin(&self, event: &CommandEvent) -> bool {
        if event.mask.is_some() {
            let mask = event.mask.unwrap().as_slice();
            if self.admins.iter().any(|r| r.is_match(mask)) {
                return true
            }
        }
        self.send_message(event.channel, "Permission denied");
        return false;
    }
}

impl Clone for IrcInterface {
    fn clone(&self) -> IrcInterface {
        return IrcInterface {
            data_out: self.data_out.clone(),
            config: self.config.clone(),
            admins: self.admins.clone()
        }
    }
}


pub struct IrcMessageEvent<'a> {
    pub client: &'a IrcInterface,
    pub command: &'a str,
    pub args: &'a [&'a str],
    pub mask: Option<&'a str>
}

pub struct CommandEvent<'a> {
    pub client: &'a IrcInterface,
    pub channel: &'a str,
    pub args: &'a [&'a str],
    pub mask: Option<&'a str>
}


impl <'a> IrcMessageEvent<'a> {
    fn new(client: &'a IrcInterface, command: &'a str, args: &'a [&'a str], mask: Option<&'a str>) -> IrcMessageEvent<'a> {
        return IrcMessageEvent {
            client: client,
            command: command,
            args: args,
            mask: mask
        }
    }
}

impl <'a> CommandEvent<'a> {
    fn new(client: &'a IrcInterface, channel: &'a str, args: &'a [&'a str], mask: Option<&'a str>) -> CommandEvent<'a> {
        return CommandEvent {
            client: client,
            channel: channel,
            args: args,
            mask: mask
        }
    }
}

#[deriving(Decodable)]
pub struct ClientConfiguration {
    pub name: String,
    pub nick: String,
    pub user: String,
    pub real_name: String,
    pub channels: Vec<String>,
    pub address: String,
    pub command_prefix: String,
    pub admins: Vec<String>
}

pub fn load_config_from_file(path: &Path) -> Result<ClientConfiguration, InitializationError> {
    let config_contents = try!(File::open(path).read_to_string());
    let client_config = try!(match json::decode::<ClientConfiguration>(config_contents.as_slice()) {
        Ok(v) => Ok(v),
        Err(e) => {
            match e {
                DecoderError::ParseError(parse_error) => match parse_error {
                    ParserError::SyntaxError(error_code, line, col) => return Err(InitializationError::from_string(format!("Syntax error ({}) on line {} column {} in {}", error_code, line, col, path.display()))),
                    ParserError::IoError(kind, desc) => return Err(InitializationError::Io(IoError{ kind: kind, desc: desc, detail: None}))
                },
                DecoderError::MissingFieldError(s) => return Err(InitializationError::from_string(format!("Field {} not found in {}", s.as_slice(), path.display()))),
                _ => Err(e)
            }
        }
    });

    return Ok(client_config)
}
