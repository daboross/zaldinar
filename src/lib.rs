#![feature(phase)]
#![feature(tuple_indexing)]

extern crate serialize;
extern crate regex;

use std::sync::{Arc, RWLock};
use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::task::TaskBuilder;

pub use errors::InitializationError;
pub use config::ClientConfiguration;
pub use interface::{CommandEvent, IrcMessageEvent, IrcInterface};
use irc::{IrcConnection, IrcMessage};

pub mod errors;
pub mod config;
pub mod interface;
mod irc;


// TODO: Store channels joined
pub struct Client {
    data_in: Receiver<Option<IrcMessage>>,
    pub interface: IrcInterface,
    commands: Arc<RWLock<HashMap<String, Vec<|&CommandEvent|:Send + Sync>>>>,
    raw_listeners: Arc<RWLock<HashMap<String, Vec<|&IrcMessageEvent|:Send + Sync>>>>,
    catch_all: Arc<RWLock<Vec<|&IrcMessageEvent|:Send + Sync>>>,
    config: Arc<ClientConfiguration>,
    irc_connection_channel: Option<(Sender<Option<IrcMessage>>, Receiver<Option<String>>)>,
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
            catch_all: Arc::new(RWLock::new(Vec::new())),
            config: rc_config,
            irc_connection_channel: Some((connection_data_out, connection_data_in))
        };

        // Add initial channel join listener
        client.add_listener("004", |event: &IrcMessageEvent| {
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
            None => return Err(InitializationError::new("Already connected")),
        };
        self.irc_connection_channel = None;

        // Send NICK and USER, the initial IRC commands. Because an IrcConnection hasn't been created to receive these yet,
        //  they will just go on hold and get sent as soon as the IrcConnection connects.
        self.interface.send_command("NICK".into_string(), &[&*self.config.nick]);
        self.interface.send_command("USER".into_string(), &[&*self.config.user, "0", "*", &*format!(":{}", self.config.real_name)]);

        try!(IrcConnection::create(self.config.address.as_slice(), connection_data_out, connection_data_in));
        self.spawn_dispatch_thread();
        return Ok(());
    }

    pub fn add_listener(&mut self, irc_command: &str, f: |&IrcMessageEvent|:Send + Sync) {
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

    pub fn add_catch_all_listener(&mut self, f: |&IrcMessageEvent|:Send + Sync) {
        let mut listeners = self.catch_all.write();
        {
            listeners.push(f);
        }
        listeners.downgrade();
    }

    pub fn add_command(&mut self, command: &str, f: |&CommandEvent|:Send + Sync) {
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
        TaskBuilder::new().named("client_dispatch_task").spawn(proc() {
            loop {
                let message = match self.data_in.recv() {
                    Some(v) => v,
                    None => break,
                };
                self.process_message(&message);
            }
        });
    }

    // Noting: This has to be a separate method from spawn_dispatch_thread, so that we can name an 'a lifetime.
    // This allows us to give the new &str slices a specific lifetime, which I don't know a way to do without making a new function.
    fn process_message<'a>(&self, message: &'a IrcMessage) {
        let shared_mask: Option<&str> = message.mask.as_ref().map(|s| &**s);
        let shared_args = message.args.iter().map(|s| &**s).collect::<Vec<&'a str>>();
        let shared_ctcp = message.ctcp.as_ref().map(|t| (t.0.as_slice(), t.1.as_slice()));

        // PING
        if message.command.as_slice().eq_ignore_ascii_case("PING") {
            self.interface.send_command("PONG".into_string(), shared_args.as_slice());
        }

        let message_event = &mut IrcMessageEvent::new(&self.interface, message.command.as_slice(), shared_args.as_slice(), shared_mask, shared_ctcp);

        // Catch all listeners
        let mut catch_all = self.catch_all.write();
        {
            for listener in catch_all.iter_mut() {
                (*listener)(message_event);
            }
        }
        catch_all.downgrade();

        // Raw listeners
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
