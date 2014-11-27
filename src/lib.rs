#![feature(globs)]

extern crate serialize;

use std::io::*;
use std::sync::{Arc, RWLock};
use std::ascii::AsciiExt;
use std::collections::HashMap;
use serialize::json;

struct IrcConnection {
    name: String,
    socket: TcpStream,
    data_out: Option<Sender<IrcMessage>>,
    data_in: Option<Receiver<String>>,
}

impl IrcConnection {
    pub fn create(name: &str, addr: &str, data_out: Sender<IrcMessage>, data_in: Receiver<String>) -> Result<(), IoError> {
        let socket = try!(TcpStream::connect(addr));
        let connection_receiving = IrcConnection {
            name: name.into_string(),
            socket: socket.clone(),
            data_out: Some(data_out),
            data_in: None
        };
        let connection_sending = IrcConnection {
            name: name.into_string(),
            socket: socket, // No need to clone a second time, as this is the last time we are using this socket
            data_out: None,
            data_in: Some(data_in)
        };
        connection_receiving.spawn_reading_thread().ok().expect("This was already setup to work");
        connection_sending.spawn_writing_thread().ok().expect("This was already setup to work");
        return Ok(())
    }

    fn spawn_reading_thread(self) -> Result<(), &'static str> {
        let data_out = match self.data_out {
            Some(ref v) => v.clone(),
            None => return Err("Can't start reading thread without data_out")
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
                let message = IrcMessage::new(self.name.clone(), command.into_string(), args_owned, possible_mask);
                data_out.send(message);
            }
        });
        return Ok(())
    }

    fn spawn_writing_thread(mut self) -> Result<(), &'static str> {
        if (&self.data_in).is_none() {
            return Err("Can't start writing thread without data_in")
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
    server: String,
    command: String,
    args: Vec<String>,
    mask: Option<String>
}

impl IrcMessage {
    fn new(server: String, command: String, args: Vec<String>, mask: Option<String>) -> IrcMessage {
        return IrcMessage {
            server: server,
            command: command,
            args: args,
            mask: mask
        }
    }
}

struct Client {
    data_out: Sender<String>,
    data_in: Receiver<IrcMessage>,
    interface: IrcInterface,
    commands: Arc<RWLock<HashMap<String, Vec<|&mut CommandEvent|:'static>>>>,
    raw_listeners: Arc<RWLock<HashMap<String, Vec<|&mut IrcMessageEvent|:'static>>>>,
    name: String,
    bot_nick: String,
    bot_username: String,
    bot_real_name: String,
    server_address: String,
    command_prefix: String,
    // TODO: Update channels with channels actually joined, not just configured.
    channels: Vec<String>,
    irc_connection_channel: Option<(Sender<IrcMessage>, Receiver<String>)>,
}

impl Client {
    pub fn new(config: ClientConfiguration) -> Client {
        let (data_out, connection_data_in) = channel();
        let (connection_data_out, data_in) = channel();
        let client = Client {
            interface: IrcInterface::new(data_out.clone()),
            data_out: data_out,
            data_in: data_in,
            commands: Arc::new(RWLock::new(HashMap::new())),
            raw_listeners: Arc::new(RWLock::new(HashMap::new())),
            name: config.name,
            bot_nick: config.nick,
            bot_username: config.user,
            bot_real_name: config.real_name,
            channels: config.channels,
            server_address: config.address,
            command_prefix: config.command_prefix,
            irc_connection_channel: Some((connection_data_out, connection_data_in))
        };
        return client;
    }

    pub fn connect(mut self) -> Result<(), &'static str> {
        let (connection_data_out, connection_data_in) = match self.irc_connection_channel {
            Some(v) => {
                self.irc_connection_channel = None;
                v
            },
            None => return Err("Already connected")
        };
        IrcConnection::create(self.name.as_slice(), self.server_address.as_slice(), connection_data_out, connection_data_in);
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
                // let mask: Option<&str> = match message.mask {
                //     Some(v) => Some(v.as_slice()),
                //     None => None
                // };
                let args_shared = &[];//message.args.iter().map(|s: &String| s.as_slice()).collect::<Vec<&str>>();
                let event = &mut IrcMessageEvent::new(&self.interface, message.command.as_slice(), args_shared.as_slice(), None);
                let mut listener_map = self.raw_listeners.write();
                {
                    let mut listeners = match listener_map.get_mut(&message.command.to_ascii_lower()) {
                        Some(v) => v,
                        None => continue
                    };

                    for listener in listeners.iter_mut() {
                        (*listener)(event);
                    }
                }
                listener_map.downgrade();
            }
        });
    }
}

pub struct IrcInterface {
    data_out: Sender<String>
}

impl IrcInterface {
    fn new(data_out: Sender<String>) -> IrcInterface {
        return IrcInterface {
            data_out: data_out
        }
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
}

        // client.add_listener("ping", |event: &mut InternalIrcEvent| {
        //     event.client.send(format!("PONG {}", event.args[0]).as_slice());
        // });
        // client.add_listener("004", |event: &mut InternalIrcEvent| {
        //     // TODO: Merge IrcClient and IrcConnection to deal with all this
        //     for channel in self.channels.iter() {
        //         event.client.send(format!("JOIN {}", channel.as_slice()).as_slice());
        //     }
        //     event.client.send("JOIN #bot");
        // });
        // client.add_listener("privmsg", |event: &mut InternalIrcEvent| {
        //     // let permitted = regex!(r"^Dabo[^!]*![^@]*@me.dabo.guru$");
        //     // let mask = event.mask.expect("PRIVMSG received without sender mask");
        //     let channel = event.args[0];
        //     if event.args[1].starts_with(format!(":{}", self.command_prefix.as_slice()).as_slice()) {
        //         let command = event.args[1].slice_from(2).into_string().to_ascii_lower();
        //         let mut command_map = self.commands.write();
        //         {
        //             let mut commands = match command_map.get_mut(&command) {
        //                 Some(v) => v,
        //                 None => return
        //             };

        //             let args = event.args.slice_from(2);
        //             let shared_self = &mut self.clone();
        //             let event = &mut CommandEvent::new(shared_self, command.as_slice(), args, event.mask);

        //             for command in commands.iter_mut() {
        //                 (*command)(event);
        //             }
        //         }
        //         command_map.downgrade();
        //     }

        // if event.args[1].eq_ignore_ascii_case(":quit") && permitted.is_match(mask) {
        //     event.client.send("QUIT :Testing.");
        // } else if event.args[1].eq_ignore_ascii_case(":raw") && permitted.is_match(mask) {
        //     event.client.send(event.args.slice_from(2).connect(" ").as_slice())
        // } else {
        //     event.client.send(format!("PRIVMSG {}", event.args.connect(" ")).as_slice());
        // }

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

#[deriving(Decodable, Encodable)]
pub struct ClientConfiguration {
    name: String,
    nick: String,
    user: String,
    real_name: String,
    channels: Vec<String>,
    address: String,
    command_prefix: String
}

pub fn load_config_from_file(path: &Path) -> Result<ClientConfiguration, String> {
    let config_contents = match File::open(path).read_to_string() {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to read config file: {}", e))
    };
    let client_config = match json::decode::<ClientConfiguration>(config_contents.as_slice()) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to decode config file: {}", e))
    };
    return Ok(client_config)
}

pub fn start_from_config(config: ClientConfiguration) {
    let mut client = Client::new(config);
    client.add_command("say", |event: &mut CommandEvent| {
        event.client.send_command("PRIVMSG".to_string(), event.args);
    });
    client.connect().ok().expect("Failed to connect!");
}
