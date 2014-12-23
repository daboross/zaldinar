use std::ascii::AsciiExt;
use std::sync;
use std::collections;

use chrono;
use fern;
use fern_macros;

use errors::InitializationError;
use plugins;
use interface;
use config;
use irc;

pub struct PluginRegister {
    commands: collections::HashMap<String, Vec<Box<Fn(&interface::CommandEvent) + Send + Sync>>>,
    ctcp_listeners: collections::HashMap<String, Vec<Box<Fn(&interface::CtcpEvent) + Send + Sync>>>,
    raw_listeners: collections::HashMap<String, Vec<Box<Fn(&interface::IrcMessageEvent) + Send + Sync>>>,
    catch_all: Vec<Box<Fn(&interface::IrcMessageEvent) + Send + Sync>>,
}

impl PluginRegister {
    pub fn new() -> PluginRegister {
        return PluginRegister {
            commands: collections::HashMap::new(),
            raw_listeners: collections::HashMap::new(),
            ctcp_listeners: collections::HashMap::new(),
            catch_all: Vec::new(),
        }
    }

    pub fn register_irc<T: Fn(&interface::IrcMessageEvent) + Send + Sync>(&mut self, irc_command: &str, f: T) {
        let boxed = box f as Box<Fn(&interface::IrcMessageEvent) + Send + Sync>;
        let command_string = irc_command.into_string().to_ascii_lower();

        // I can't use a match here because then I would be borrowing listener_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if self.raw_listeners.contains_key(&command_string) {
            self.raw_listeners.get_mut(&command_string).expect("Honestly, this won't happen.").push(boxed);
        } else {
            self.raw_listeners.insert(command_string, vec!(boxed));
        }
    }

    pub fn register_ctcp<T: Fn(&interface::CtcpEvent) + Send + Sync>(&mut self, ctcp_command: &str, f: T) {
        let boxed = box f as Box<Fn(&interface::CtcpEvent) + Send + Sync>;
        let command_string = ctcp_command.into_string().to_ascii_lower();

        // I can't use a match here because then I would be borrowing listener_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if self.ctcp_listeners.contains_key(&command_string) {
            self.ctcp_listeners.get_mut(&command_string).expect("Honestly, this won't happen.").push(boxed);
        } else {
            self.ctcp_listeners.insert(command_string, vec!(boxed));
        }
    }


    pub fn register_catch_all<T: Fn(&interface::IrcMessageEvent) + Send + Sync>(&mut self, f: T) {
        self.catch_all.push(box f as Box<Fn(&interface::IrcMessageEvent) + Send + Sync>);
    }

    pub fn register_command<T: Fn(&interface::CommandEvent) + Send + Sync>(&mut self, command: &str, f: T) {
        let boxed = box f as Box<Fn(&interface::CommandEvent) + Send + Sync>;
        let command_lower = command.into_string().to_ascii_lower();

        // I can't use a match here because then I would be borrowing the command_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if self.commands.contains_key(&command_lower) {
            self.commands.get_mut(&command_lower).expect("Honestly, this won't happen.").push(boxed);
        } else {
            self.commands.insert(command_lower, vec!(boxed));
        }
    }
}

pub struct ClientState {
    pub nick: String,
    pub channels: Vec<String>,
}

impl ClientState {
    pub fn new(nick: String) -> ClientState {
        return ClientState {
            nick: nick,
            channels: Vec::new(),
        };
    }
}

pub struct Client {
    pub plugins: sync::RWLock<PluginRegister>,
    pub config: config::ClientConfiguration,
    pub state: sync::RWLock<ClientState>,
}

impl Client {
    pub fn new(plugins: PluginRegister, config: config::ClientConfiguration) -> Client {
        let state = sync::RWLock::new(ClientState::new(config.nick.clone()));
        return Client {
            plugins: sync::RWLock::new(plugins),
            config: config,
            state: state,
        }
    }
}

/// This allows access to configuration fields directly on Client
impl Deref<config::ClientConfiguration> for Client {
    fn deref<'a>(&'a self) -> &'a config::ClientConfiguration {
        return &self.config;
    }
}

pub struct Dispatch {
    interface: interface::IrcInterface,
    state: sync::Arc<Client>,
    data_in: Receiver<Option<irc::IrcMessage>>,
}

impl Dispatch {
    fn new(interface: interface::IrcInterface, state: sync::Arc<Client>, data_in: Receiver<Option<irc::IrcMessage>>) -> Dispatch {
        return Dispatch {
            interface: interface,
            state: state,
            data_in: data_in,
        };
    }

    fn start_dispatch_loop(self) {
        loop {
            let message = match self.data_in.recv() {
                Some(v) => v,
                None => break,
            };
            self.process_message(&message);
        }
    }

    // Noting: This has to be a separate method from spawn_dispatch_thread, so that we can name an 'a lifetime.
    // This allows us to give the new &str slices a specific lifetime, which I don't know a way to do without making a new function.
    fn process_message<'a>(&self, message: &'a irc::IrcMessage) {
        let plugins = self.state.plugins.read();

        let shared_mask = &interface::IrcMask::from_internal(&message.mask);
        let shared_args = message.args.iter().map(|s| s.as_slice()).collect::<Vec<&'a str>>();
        let shared_ctcp = message.ctcp.as_ref().map(|t| (t.0.as_slice(), t.1.as_slice()));
        let shared_channel = message.channel.as_ref().map(|s| s.as_slice());

        // PING
        if message.command.as_slice().eq_ignore_ascii_case("PING") {
            self.interface.send_command("PONG".into_string(), shared_args.as_slice());
        }

        let message_event = &mut interface::IrcMessageEvent::new(&self.interface, message.command.as_slice(), shared_args.as_slice(), shared_mask, shared_ctcp, shared_channel);

        // Catch all listeners
        for listener in plugins.catch_all.iter() {
            listener.call((message_event,));
        }

        // Raw listeners
        match plugins.raw_listeners.get(&message.command.to_ascii_lower()) {
            Some(list) => {
                for listener in list.iter() {
                    listener.call((message_event,));
                }
            },
            None => (),
        }

        if message.command.as_slice().eq_ignore_ascii_case("PRIVMSG") {
            let channel = shared_channel.unwrap(); // Always exists for PRIVMSG

            // CTCP
            match message.ctcp {
                Some(ref t) => {
                    let ctcp_event = interface::CtcpEvent::new(&self.interface, message.args[0].as_slice(), t.0.as_slice(), t.1.as_slice(), shared_mask);

                    match plugins.ctcp_listeners.get(&ctcp_event.command.to_ascii_lower()) {
                        Some(list) => {
                            for ctcp_listener in list.iter() {
                                ctcp_listener.call((&ctcp_event,));
                            }
                        },
                        None => (),
                    }
                },
                None => (),
            }

            // Commands
            let command_prefix = format!(":{}", self.state.command_prefix.as_slice());

            // This checks for the command prefix, commands typed like '.command_name args'
            if shared_args[1].starts_with(command_prefix.as_slice()) {
                let command = shared_args[1].slice_from(command_prefix.len());
                let args = shared_args.slice_from(2);
                self.dispatch_command(&plugins, command, channel, args, shared_mask);
            } else {
                // This checks for someone typing commands like 'BotName, command_name args'
                // We store whether or not a command was matched in a variable so that we can use it below.
                // We can't just put the below in one of the None => branches, because there are multiple instances of them.
                let command_matched = match regex!(r"^:([^\s]+?)[:;,]?\s+(.+)$").captures(shared_args.slice_from(1).connect(" ").as_slice()) {
                    Some(captures) => {
                        if captures.at(1) == Some(self.state.state.read().nick.as_slice()) {
                            match captures.at(2) {
                                Some(args_str) => {
                                    let split = args_str.split(' ').collect::<Vec<&str>>();
                                    let command = split[0];
                                    let args = split.slice_from(1);
                                    self.dispatch_command(&plugins, command, channel, args, shared_mask);
                                    true
                                },
                                None => false,
                            }
                        } else {
                            false
                        }
                    },
                    None => false,
                };

                // This checks for commands in a private message, where a prefix isn't required
                // People can just say 'command args' in a private message.
                // If the channel is the sender's nick, the message is being sent in a private message.
                if !command_matched && shared_mask.nick() == Some(channel) {
                    let command = shared_args[1].slice_from(1); // slice_from(1) to remove the `:` at the beginning of privmsg content.
                    let args = shared_args.slice_from(2);
                    self.dispatch_command(&plugins, command, channel, args, shared_mask);
                }
            }
        }
    }

    fn dispatch_command(&self, plugins: &sync::RWLockReadGuard<PluginRegister>, command: &str, channel: &str, args: &[&str], mask: &interface::IrcMask) {
        match plugins.commands.get(&command.to_ascii_lower()) {
            Some(list) => {
                let command_event = &mut interface::CommandEvent::new(&self.interface, channel, args, mask);
                for closure in list.iter() {
                    closure.call((command_event,));
                }
            },
            None => (),
        }
    }
}

pub fn run(config: config::ClientConfiguration) -> Result<(), InitializationError> {
    run_with_plugins(config, PluginRegister::new())
}

pub fn run_with_plugins(config: config::ClientConfiguration, mut plugins: PluginRegister) -> Result<(), InitializationError> {
    // Register built-in plugins
    plugins::register_plugins(&mut plugins);

    let logger = sync::Arc::new(try!(fern::LoggerConfig {
        format: box |msg: &str, level: &fern::Level| {
            return format!("[{}][{}] {}", chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"), level, msg);
        },
        output: vec![fern::OutputConfig::Stdout, fern::OutputConfig::File(Path::new("zaldinar.log"))],
        level: fern::Level::Debug,
    }.into_logger()));

    fern_macros::init_thread_logger(logger.clone());

    let client = sync::Arc::new(Client::new(plugins, config));

    let (data_out, connection_data_in) = channel();
    let (connection_data_out, data_in) = channel();

    let interface = try!(interface::IrcInterface::new(data_out, client.clone()));


    // Send NICK and USER, the initial IRC commands. Because an IrcConnection hasn't been created to receive these yet,
    //  they will just go on hold and get sent as soon as the IrcConnection connects.
    interface.send_command("NICK".into_string(), &[client.nick.as_slice()]);
    interface.send_command("USER".into_string(), &[client.user.as_slice(), "0", "*", format!(":{}", client.real_name).as_slice()]);

    try!(irc::IrcConnection::create(client.address.as_slice(), connection_data_out, connection_data_in, logger.clone(), client.clone()));

    let dispatch = Dispatch::new(interface, client, data_in);

    // This statement will run until the bot exists
    dispatch.start_dispatch_loop();

    return Ok(());
}
