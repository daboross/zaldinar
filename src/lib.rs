extern crate serialize;
extern crate regex;

use std::io::{TcpStream, IoError, BufferedReader};
use std::sync::{Arc, RWLock};
use std::ascii::AsciiExt;
use std::collections::HashMap;

use errors::InitializationError;
use config::ClientConfiguration;
use interface::{CommandEvent, IrcMessageEvent, IrcInterface};

pub mod errors;
pub mod config;
pub mod interface;

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
        let mut client = Client {
            interface: try!(IrcInterface::new(data_out, rc_config.clone())),
            data_in: data_in,
            commands: Arc::new(RWLock::new(HashMap::new())),
            raw_listeners: Arc::new(RWLock::new(HashMap::new())),
            config: rc_config,
            irc_connection_channel: Some((connection_data_out, connection_data_in))
        };

        // Add initial channel join listener
        client.add_listener("004", |event: &mut IrcMessageEvent| {
            let nickserv = &event.client.config.nickserv;
            if nickserv.enabled {
                if nickserv.account.len() != 0 {
                    event.client.send_message(nickserv.name.as_slice(), format!("{} {} {}", nickserv.command, nickserv.account, nickserv.password).as_slice());
                } else {
                    event.client.send_message(nickserv.name.as_slice(), format!("{} {}", nickserv.command, nickserv.password).as_slice());
                }
            }

            for channel in event.client.config.channels.iter() {
                event.client.send_command("JOIN".into_string(), &[channel.as_slice()]);
            }
        });

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
