use std::ascii::AsciiExt;
use std::sync;
use std::sync::mpsc;
use std::ops;
use std::collections;

use chrono;
use fern;

use errors::InitializationError;
use plugins;
use interface;
use config;
use dispatch;
use irc;
use events;
#[cfg(target_os = "linux")]
use filewatch;

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

    pub fn register_irc<T>(&mut self, irc_command: &str, f: T)
            where T: Fn(&events::MessageEvent) + Send + Sync {
        let boxed = sync::Arc::new(box f as MessageListener);
        let command_string = irc_command.to_string().to_ascii_lowercase();

        // I can't use a match here because then I would be borrowing listener_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if self.raw_listeners.contains_key(&command_string) {
            self.raw_listeners.get_mut(&command_string).expect("Confirmed above").push(boxed);
        } else {
            self.raw_listeners.insert(command_string, vec!(boxed));
        }
    }

    pub fn register_ctcp<T>(&mut self, ctcp_command: &str, f: T)
            where T: Fn(&events::CtcpEvent) + Send + Sync {
        let boxed = sync::Arc::new(box f as CtcpListener);
        let command_string = ctcp_command.to_string().to_ascii_lowercase();

        // I can't use a match here because then I would be borrowing listener_map mutably twice:
        // Once for the match statement, and a second time inside the None branch
        if self.ctcp_listeners.contains_key(&command_string) {
            self.ctcp_listeners.get_mut(&command_string).expect("Confirmed above").push(boxed);
        } else {
            self.ctcp_listeners.insert(command_string, vec!(boxed));
        }
    }


    pub fn register_catch_all<T>(&mut self, f: T)
            where T: Fn(&events::MessageEvent) + Send + Sync {
        self.catch_all.push(sync::Arc::new(box f as MessageListener));
    }

    pub fn register_command<T>(&mut self, command: &str, f: T)
            where T: Fn(&events::CommandEvent) + Send + Sync {
        let boxed = sync::Arc::new(box f as CommandListener);
        let command_lower = command.to_string().to_ascii_lowercase();

        // I can't use a match here because then I would be borrowing the command_map mutably
        // twice:
        // Once for the match statement, and a second time inside the None branch
        if self.commands.contains_key(&command_lower) {
            self.commands.get_mut(&command_lower).expect("Confirmed above").push(boxed);
        } else {
            self.commands.insert(command_lower, vec!(boxed));
        }
    }
}

#[derive(Copy, Clone)]
pub enum ExecutingState {
    Running,
    Done,
    Restart,
}

pub struct ClientState {
    pub nick: String,
    pub channels: Vec<String>,
    /// This is a marker for what the bot should do after the main program exits.
    /// - The main function will just be re-run if this is still "Running".
    /// - The bot will exit if this is "Done".
    /// - The bot will restart using exec (running using a new binary if there is one) if this is
    ///   "restart".
    pub done_executing: ExecutingState,
}

impl ClientState {
    pub fn new(nick: String) -> ClientState {
        return ClientState {
            nick: nick,
            channels: Vec::new(),
            done_executing: ExecutingState::Running,
        };
    }
}

pub struct Client {
    pub plugins: sync::RwLock<PluginRegister>,
    pub config: config::ClientConfiguration,
    pub state: sync::RwLock<ClientState>,
}

impl Client {
    pub fn new(plugins: PluginRegister, config: config::ClientConfiguration) -> Client {
        let state = sync::RwLock::new(ClientState::new(config.nick.clone()));
        return Client {
            plugins: sync::RwLock::new(plugins),
            config: config,
            state: state,
        }
    }
}

/// This allows access to configuration fields directly on Client
impl ops::Deref for Client {
    type Target = config::ClientConfiguration;

    fn deref<'a>(&'a self) -> &'a config::ClientConfiguration {
        return &self.config;
    }
}

#[cfg(target_os = "linux")]
fn start_file_watch(client: &sync::Arc<Client>, interface: &interface::IrcInterface) {
    if client.watch_binary {
        if let Err(e) = filewatch::watch_binary(interface.clone()) {
            warning!("Failed to start binary watch thread: {}", e);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn start_file_watch(_client: &sync::Arc<Client>, _interface: &interface::IrcInterface) {
    // TODO: Maybe support this?
}

pub fn run(config: config::ClientConfiguration) -> Result<ExecutingState, InitializationError> {
    run_with_plugins(config, PluginRegister::new())
}

pub fn run_with_plugins(config: config::ClientConfiguration, mut plugins: PluginRegister)
        -> Result<ExecutingState, InitializationError> {
    // Register built-in plugins
    plugins::register_plugins(&mut plugins);

    let logger = sync::Arc::new(try!(fern::LoggerConfig {
        format: box |msg: &str, level: &fern::Level| {
            return format!("[{}][{:?}] {}", chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"),
                level, msg);
        },
        output: vec![fern::OutputConfig::Stdout, fern::OutputConfig::File(
                                                    Path::new(&config.log_file))],
        level: fern::Level::Info,
    }.into_logger()));

    fern::local::set_thread_logger(logger.clone());

    let client = sync::Arc::new(Client::new(plugins, config));

    let (data_out, connection_data_in) = mpsc::channel();
    let (connection_data_out, data_in) = mpsc::channel();

    let interface = try!(interface::IrcInterface::new(data_out, client.clone()));

    // Load file watcher
    start_file_watch(&client, &interface);

    // Send PASS, NICK and USER, the initial IRC commands. Because an IrcConnection hasn't been
    // created to receive these yet, they will just go on hold and get sent as soon as the
    // IrcConnection connects.
    if let Some(ref pass) = client.password {
        interface.send_command("PASS".to_string(), &[&pass]);
    }
    interface.send_command("NICK".to_string(), &[&client.nick]);
    interface.send_command("USER".to_string(), &[&client.user, "0", "*",
        &format!(":{}", client.real_name)]);

    try!(irc::IrcConnection::create(&client.address, connection_data_out,
        connection_data_in, logger.clone(), client.clone()));

    // Create dispatch, and start the worker threads for plugin execution
    let dispatch = dispatch::Dispatch::new(interface, client.clone(), data_in, logger);

    // This statement will run until the bot exists
    if let Err(..) = dispatch.start_dispatch_loop() {
        severe!("Dispatch loop panicked!");
    }

    let done = {
        let state = try!(client.state.read());
        state.done_executing
    };

    // Possibly drop the thread logger
    fern::local::set_thread_logger(sync::Arc::new(box fern::NullLogger as fern::BoxedLogger));

    return Ok(done);
}
