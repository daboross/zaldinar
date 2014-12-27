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
use dispatch;
use irc;
use events;

pub type CommandListener = Box<Fn(&events::CommandEvent) + Sync + Send>;
pub type CtcpListener = Box<Fn(&events::CtcpEvent) + Sync + Send>;
pub type MessageListener = Box<Fn(&events::MessageEvent) + Sync + Send>;

pub struct PluginRegister {
    pub commands: collections::HashMap<String, Vec<sync::Arc<CommandListener>>>,
    pub ctcp_listeners: collections::HashMap<String, Vec<sync::Arc<CtcpListener>>>,
    pub raw_listeners: collections::HashMap<String, Vec<sync::Arc<MessageListener>>>,
    pub catch_all: Vec<sync::Arc<MessageListener>>,
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

    pub fn register_irc<T: Fn(&events::MessageEvent) + Send + Sync>(&mut self, irc_command: &str, f: T) {
        let boxed = sync::Arc::new(box f as MessageListener);
        let command_string = irc_command.to_string().to_ascii_lower();

        // I can't use a match here because then I would be borrowing listener_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if self.raw_listeners.contains_key(&command_string) {
            self.raw_listeners.get_mut(&command_string).expect("Honestly, this won't happen.").push(boxed);
        } else {
            self.raw_listeners.insert(command_string, vec!(boxed));
        }
    }

    pub fn register_ctcp<T: Fn(&events::CtcpEvent) + Send + Sync>(&mut self, ctcp_command: &str, f: T) {
        let boxed = sync::Arc::new(box f as CtcpListener);
        let command_string = ctcp_command.to_string().to_ascii_lower();

        // I can't use a match here because then I would be borrowing listener_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if self.ctcp_listeners.contains_key(&command_string) {
            self.ctcp_listeners.get_mut(&command_string).expect("Honestly, this won't happen.").push(boxed);
        } else {
            self.ctcp_listeners.insert(command_string, vec!(boxed));
        }
    }


    pub fn register_catch_all<T: Fn(&events::MessageEvent) + Send + Sync>(&mut self, f: T) {
        self.catch_all.push(sync::Arc::new(box f as MessageListener));
    }

    pub fn register_command<T: Fn(&events::CommandEvent) + Send + Sync>(&mut self, command: &str, f: T) {
        let boxed = sync::Arc::new(box f as CommandListener);
        let command_lower = command.to_string().to_ascii_lower();

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
    /// This is a marker for if the bot exited on purpose. The bot will restart if this is still true after exiting.
    pub done_executing: bool,
}

impl ClientState {
    pub fn new(nick: String) -> ClientState {
        return ClientState {
            nick: nick,
            channels: Vec::new(),
            done_executing: false,
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

pub fn run(config: config::ClientConfiguration) -> Result<bool, InitializationError> {
    run_with_plugins(config, PluginRegister::new())
}

pub fn run_with_plugins(config: config::ClientConfiguration, mut plugins: PluginRegister) -> Result<bool, InitializationError> {
    // Register built-in plugins
    plugins::register_plugins(&mut plugins);

    let logger = sync::Arc::new(try!(fern::LoggerConfig {
        format: box |msg: &str, level: &fern::Level| {
            return format!("[{}][{}] {}", chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"), level, msg);
        },
        output: vec![fern::OutputConfig::Stdout, fern::OutputConfig::File(Path::new(config.log_file.as_slice()))],
        level: fern::Level::Debug,
    }.into_logger()));

    fern_macros::init_thread_logger(logger.clone());

    let client = sync::Arc::new(Client::new(plugins, config));

    let (data_out, connection_data_in) = channel();
    let (connection_data_out, data_in) = channel();

    let interface = try!(interface::IrcInterface::new(data_out, client.clone()));


    // Send PASS, NICK and USER, the initial IRC commands. Because an IrcConnection hasn't been created to receive these yet,
    //  they will just go on hold and get sent as soon as the IrcConnection connects.
    if let Some(ref pass) = client.password {
        interface.send_command("PASS".to_string(), &[pass.as_slice()]);
    }
    interface.send_command("NICK".to_string(), &[client.nick.as_slice()]);
    interface.send_command("USER".to_string(), &[client.user.as_slice(), "0", "*", format!(":{}", client.real_name).as_slice()]);

    try!(irc::IrcConnection::create(client.address.as_slice(), connection_data_out, connection_data_in, logger.clone(), client.clone()));

    // Create dispatch, and start the worker threads for plugin execution
    let dispatch = dispatch::Dispatch::new(interface, client.clone(), data_in, logger);

    // This statement will run until the bot exists
    if let Err(..) = dispatch.start_dispatch_loop() {
        severe!("Dispatch loop panicked!");
    }

    let done = {
        let state = client.state.read();
        state.done_executing
    };

    return Ok(done);
}
