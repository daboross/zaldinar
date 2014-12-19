#![feature(phase)]
#![feature(unboxed_closures)]

extern crate serialize;
extern crate regex;
extern crate chrono;
extern crate fern;
#[phase(plugin, link)]
extern crate fern_macros;
#[phase(plugin)]
extern crate regex_macros;

use std::sync::{Arc, RWLock};
use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::task::TaskBuilder;

pub use errors::InitializationError;
pub use config::ClientConfiguration;
pub use interface::{CommandEvent, IrcMessageEvent, CtcpEvent, IrcInterface};
use irc::{IrcConnection, IrcMessage};

pub mod errors;
pub mod config;
pub mod interface;
mod irc;
mod plugins;

pub fn get_version() -> String {
    return format!("{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));
}

// TODO: Store channels joined
// TODO: Make a 'ClientState' object which holds current state like channels joined, current nick, etc. and share it in a Arc<RWLock<>> between all interfaces and connections
pub struct Client {
    data_in: Receiver<Option<IrcMessage>>,
    pub interface: IrcInterface,
    commands: Arc<RWLock<HashMap<String, Vec<Box<Fn(&CommandEvent) + Send + Sync>>>>>,
    ctcp_listeners: Arc<RWLock<HashMap<String, Vec<Box<Fn(&CtcpEvent) + Send + Sync>>>>>,
    raw_listeners: Arc<RWLock<HashMap<String, Vec<Box<Fn(&IrcMessageEvent) + Send + Sync>>>>>,
    catch_all: Arc<RWLock<Vec<Box<Fn(&IrcMessageEvent) + Send + Sync>>>>,
    config: Arc<ClientConfiguration>,
    irc_connection_channel: Option<(Sender<Option<IrcMessage>>, Receiver<Option<String>>)>,
    logger: Arc<Box<fern::Logger + Sync + Send>>,
}

impl Client {
    pub fn new(config: ClientConfiguration) -> Result<Client, InitializationError> {
        let rc_config = Arc::new(config);
        let (data_out, connection_data_in) = channel();
        let (connection_data_out, data_in) = channel();
        let logger = try!(fern::LoggerConfig {
            format: box |msg: &str, level: &fern::Level| {
                return format!("[{}][{}] {}", chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"), level, msg);
            },
            output: vec![fern::OutputConfig::Stdout, fern::OutputConfig::File(Path::new("zaldinar.log"))],
            level: fern::Level::Debug,
        }.into_logger());
        let mut client = Client {
            interface: try!(IrcInterface::new(data_out, rc_config.clone())),
            data_in: data_in,
            commands: Arc::new(RWLock::new(HashMap::new())),
            raw_listeners: Arc::new(RWLock::new(HashMap::new())),
            ctcp_listeners: Arc::new(RWLock::new(HashMap::new())),
            catch_all: Arc::new(RWLock::new(Vec::new())),
            config: rc_config,
            irc_connection_channel: Some((connection_data_out, connection_data_in)),
            logger: Arc::new(logger),
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

        // Add built-in plugins to the Client
        plugins::register_plugins(&mut client);

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

        try!(IrcConnection::create(self.config.address.as_slice(), connection_data_out, connection_data_in, self.logger.clone()));
        self.spawn_dispatch_thread();
        return Ok(());
    }

    pub fn add_listener<T: Fn(&IrcMessageEvent) + Send + Sync>(&mut self, irc_command: &str, f: T) {
        let boxed = box f as Box<Fn(&IrcMessageEvent) + Send + Sync>;
        let command_string = irc_command.into_string().to_ascii_lower();

        let mut listener_map = self.raw_listeners.write();
        // I can't use a match here because then I would be borrowing listener_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if listener_map.contains_key(&command_string) {
            listener_map.get_mut(&command_string).expect("Honestly, this won't happen.").push(boxed);
        } else {
            listener_map.insert(command_string, vec!(boxed));
        }
    }

    pub fn add_ctcp_listener<T: Fn(&CtcpEvent) + Send + Sync>(&mut self, ctcp_command: &str, f: T) {
        let boxed = box f as Box<Fn(&CtcpEvent) + Send + Sync>;
        let command_string = ctcp_command.into_string().to_ascii_lower();

        let mut listener_map = self.ctcp_listeners.write();
        // I can't use a match here because then I would be borrowing listener_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if listener_map.contains_key(&command_string) {
            listener_map.get_mut(&command_string).expect("Honestly, this won't happen.").push(boxed);
        } else {
            listener_map.insert(command_string, vec!(boxed));
        }
    }


    pub fn add_catch_all_listener<T: Fn(&IrcMessageEvent) + Send + Sync>(&mut self, f: T) {
        self.catch_all.write().push(box f as Box<Fn(&IrcMessageEvent) + Send + Sync>);
    }

    pub fn add_command<T: Fn(&CommandEvent) + Send + Sync>(&mut self, command: &str, f: T) {
        let boxed = box f as Box<Fn(&CommandEvent) + Send + Sync>;
        let command_lower = command.into_string().to_ascii_lower();

        let mut command_map = self.commands.write();
        // I can't use a match here because then I would be borrowing the command_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if command_map.contains_key(&command_lower) {
            command_map.get_mut(&command_lower).expect("Honestly, this won't happen.").push(boxed);
        } else {
            command_map.insert(command_lower, vec!(boxed));
        }
    }

    fn spawn_dispatch_thread(self) {
        TaskBuilder::new().named("client_dispatch_task").spawn(move || {
            fern_macros::init_thread_logger(self.logger.clone());
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
        // let shared_mask: Option<&str> = message.mask.as_ref().map(|s| &**s);
        let shared_mask = &interface::IrcMask::from_internal(&message.mask);
        let shared_args = message.args.iter().map(|s| &**s).collect::<Vec<&'a str>>();
        let shared_ctcp = message.ctcp.as_ref().map(|t| (t.0.as_slice(), t.1.as_slice()));

        // PING
        if message.command.as_slice().eq_ignore_ascii_case("PING") {
            self.interface.send_command("PONG".into_string(), shared_args.as_slice());
        }

        let message_event = &mut IrcMessageEvent::new(&self.interface, message.command.as_slice(), shared_args.as_slice(), shared_mask, shared_ctcp);

        // Catch all listeners
        {
            let catch_all = self.catch_all.read();
            for listener in catch_all.iter() {
                listener.call((message_event,));
            }
        }

        // Raw listeners
        { // New scope so that listener_map will go out of scope after we use it
            let listener_map = self.raw_listeners.read();

            let listeners = listener_map.get(&message.command.to_ascii_lower());
            match listeners {
                Some(list) => {
                    for listener in list.iter() {
                        listener.call((message_event,));
                    }
                },
                None => (),
            }
        }

        if message.command.as_slice().eq_ignore_ascii_case("PRIVMSG") {

            // CTCP
            match message.ctcp {
                Some(ref t) => {
                    let ctcp_event = CtcpEvent::new(&self.interface, message.args[0].as_slice(), t.0.as_slice(), t.1.as_slice(), shared_mask);
                    { // New scope so that ctcp_map will go out of scope after we use it
                        let ctcp_map = self.ctcp_listeners.read();
                        let ctcp_listeners = ctcp_map.get(&ctcp_event.command.to_ascii_lower());
                        match ctcp_listeners {
                            Some(list) => {
                                for ctcp_listener in list.iter() {
                                    ctcp_listener.call((&ctcp_event,));
                                }
                            },
                            None => (),
                        }
                    }
                },
                None => (),
            }


            // Commands
            let channel = shared_args[0];
            let prefix = format!(":{}", self.config.command_prefix.as_slice());
            if shared_args[1].starts_with(prefix.as_slice()) {
                let command = shared_args[1].slice_from(prefix.len()).into_string().to_ascii_lower();
                { // New scope so that command_map will go out of scope after we use it
                    let command_map = self.commands.read();
                    let commands = command_map.get(&command);
                    match commands {
                        Some(list) => {
                            let args = shared_args.slice_from(2);
                            let command_event = &mut CommandEvent::new(&self.interface, channel, args, shared_mask);
                            for command in list.iter() {
                                command.call((command_event,));
                            }
                        },
                        None => (),
                    }
                }
            }
        }
    }
}
