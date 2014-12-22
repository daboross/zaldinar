use std::ascii::AsciiExt;
use std::sync;
use std::collections;
use std::task;

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

    fn spawn_dispatch_thread(self, logger: sync::Arc<Box<fern::Logger + Sync + Send>>) {
        task::TaskBuilder::new().named("client_dispatch_task").spawn(move || {
            fern_macros::init_thread_logger(logger);
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
    fn process_message<'a>(&self, message: &'a irc::IrcMessage) {
        let plugins = self.state.plugins.read();

        let shared_mask = &interface::IrcMask::from_internal(&message.mask);
        let shared_args = message.args.iter().map(|s| &**s).collect::<Vec<&'a str>>();
        let shared_ctcp = message.ctcp.as_ref().map(|t| (t.0.as_slice(), t.1.as_slice()));

        // PING
        if message.command.as_slice().eq_ignore_ascii_case("PING") {
            self.interface.send_command("PONG".into_string(), shared_args.as_slice());
        }

        let message_event = &mut interface::IrcMessageEvent::new(&self.interface, message.command.as_slice(), shared_args.as_slice(), shared_mask, shared_ctcp);

        // Catch all listeners
        {
            for listener in plugins.catch_all.iter() {
                listener.call((message_event,));
            }
        }

        // Raw listeners
        { // New scope so that listener_map will go out of scope after we use it
            let listeners = plugins.raw_listeners.get(&message.command.to_ascii_lower());
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
                    let ctcp_event = interface::CtcpEvent::new(&self.interface, message.args[0].as_slice(), t.0.as_slice(), t.1.as_slice(), shared_mask);
                    { // New scope so that ctcp_map will go out of scope after we use it
                        let ctcp_listeners = plugins.ctcp_listeners.get(&ctcp_event.command.to_ascii_lower());
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
            let prefix = format!(":{}", self.state.command_prefix.as_slice());
            if shared_args[1].starts_with(prefix.as_slice()) {
                let command = shared_args[1].slice_from(prefix.len()).into_string().to_ascii_lower();
                { // New scope so that command_map will go out of scope after we use it
                    let commands = plugins.commands.get(&command);
                    match commands {
                        Some(list) => {
                            let args = shared_args.slice_from(2);
                            let command_event = &mut interface::CommandEvent::new(&self.interface, channel, args, shared_mask);
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

pub fn create_and_connect(config: config::ClientConfiguration) -> Result<(), InitializationError> {
    cac_with_plugin_register(config, PluginRegister::new())
}

pub fn cac_with_plugin_register(config: config::ClientConfiguration, mut plugins: PluginRegister) -> Result<(), InitializationError> {
        // Add built-in plugins to the Client
        plugins::register_plugins(&mut plugins);

        let client = sync::Arc::new(Client::new(plugins, config));

        let (data_out, connection_data_in) = channel();
        let (connection_data_out, data_in) = channel();

        let interface = try!(interface::IrcInterface::new(data_out, client.clone()));

        let logger = sync::Arc::new(try!(fern::LoggerConfig {
            format: box |msg: &str, level: &fern::Level| {
                return format!("[{}][{}] {}", chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"), level, msg);
            },
            output: vec![fern::OutputConfig::Stdout, fern::OutputConfig::File(Path::new("zaldinar.log"))],
            level: fern::Level::Debug,
        }.into_logger()));

        // Send NICK and USER, the initial IRC commands. Because an IrcConnection hasn't been created to receive these yet,
        //  they will just go on hold and get sent as soon as the IrcConnection connects.
        interface.send_command("NICK".into_string(), &[client.nick.as_slice()]);
        interface.send_command("USER".into_string(), &[client.user.as_slice(), "0", "*", format!(":{}", client.real_name).as_slice()]);

        try!(irc::IrcConnection::create(client.address.as_slice(), connection_data_out, connection_data_in, logger.clone()));
        Dispatch::new(interface.clone(), client, data_in).spawn_dispatch_thread(logger);

        return Ok(());
}
