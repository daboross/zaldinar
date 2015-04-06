use std::ascii::AsciiExt;
use std::sync;
use std::ops;
use std::collections;
use std::collections::hash_map;

use config;
use irc;
use events;

pub type CommandListener = Box<Fn(&events::CommandEvent) + Sync + Send>;
pub type CtcpListener = Box<Fn(&events::CtcpEvent) + Sync + Send>;
pub type MessageListener = Box<Fn(&events::MessageEvent) + Sync + Send>;

pub struct PluginRegister {
    pub commands: collections::HashMap<String, sync::Arc<CommandListener>>,
    pub admin_commands: collections::HashMap<String, sync::Arc<CommandListener>>,
    pub ctcp_listeners: collections::HashMap<String, Vec<sync::Arc<CtcpListener>>>,
    pub raw_listeners: collections::HashMap<String, Vec<sync::Arc<MessageListener>>>,
    pub catch_all: Vec<sync::Arc<MessageListener>>,
}

impl PluginRegister {
    pub fn new() -> PluginRegister {
        return PluginRegister {
            commands: collections::HashMap::new(),
            admin_commands: collections::HashMap::new(),
            raw_listeners: collections::HashMap::new(),
            ctcp_listeners: collections::HashMap::new(),
            catch_all: Vec::new(),
        }
    }

    pub fn register_irc<T>(&mut self, irc_command: &str, f: T)
            where T: Fn(&events::MessageEvent) + Send + Sync + 'static {
        let boxed = sync::Arc::new(Box::new(f) as MessageListener);
        let command_string = irc_command.to_string().to_ascii_lowercase();

        match self.raw_listeners.entry(command_string) {
            hash_map::Entry::Occupied(mut e) => e.get_mut().push(boxed),
            hash_map::Entry::Vacant(e) => drop(e.insert(vec!(boxed))),
        }
    }

    pub fn register_ctcp<T>(&mut self, ctcp_command: &str, f: T)
            where T: Fn(&events::CtcpEvent) + Send + Sync + 'static {
        let boxed = sync::Arc::new(Box::new(f) as CtcpListener);
        let command_string = ctcp_command.to_string().to_ascii_lowercase();

        match self.ctcp_listeners.entry(command_string) {
            hash_map::Entry::Occupied(mut e) => e.get_mut().push(boxed),
            hash_map::Entry::Vacant(e) => drop(e.insert(vec!(boxed))),
        }
    }


    pub fn register_catch_all<T>(&mut self, f: T)
            where T: Fn(&events::MessageEvent) + Send + Sync + 'static {
        self.catch_all.push(sync::Arc::new(Box::new(f) as MessageListener));
    }

    pub fn register_command<T>(&mut self, command: &str, f: T)
            where T: Fn(&events::CommandEvent) + Send + Sync + 'static {
        let boxed = sync::Arc::new(Box::new(f) as CommandListener);
        let command_lower = command.to_string().to_ascii_lowercase();

        match self.commands.entry(command_lower) {
            hash_map::Entry::Occupied(mut e) => {
                warn!("Replacing already registered command {} with newly registered command.",
                    command.to_ascii_lowercase());
                e.insert(boxed);
            },
            hash_map::Entry::Vacant(e) => drop(e.insert(boxed)),
        }
    }

    pub fn register_admin_command<T>(&mut self, command: &str, f: T)
            where T: Fn(&events::CommandEvent) + Send + Sync + 'static {
        let boxed = sync::Arc::new(Box::new(f) as CommandListener);
        let command_lower = command.to_string().to_ascii_lowercase();

        match self.admin_commands.entry(command_lower) {
            hash_map::Entry::Occupied(mut e) => {
                warn!("Replacing already registered command {} with newly registered command.",
                    command.to_ascii_lowercase());
                e.insert(boxed);
            },
            hash_map::Entry::Vacant(e) => drop(e.insert(boxed)),
        }
    }
}

#[derive(Copy, Clone)]
pub enum ExecutingState {
    /// Marker for when the bot is still running - if the bot exits with this, the start function
    /// will just be re-run.
    Running,
    /// The bot is done executing - no restart will be attempted.
    Done,
    /// The bot should be restarted just by running the start() function again - the process will
    /// not be restarted, the bot will.
    RestartNoExec,
    /// The bot will be restarted using exec() on the executable binary if possible, or if that
    /// isn't supported on this platform, it will just be restarted within the same process.
    RestartTryExec,
    /// Restart using exec(), or exit the process  if exec isn't supported, this will print an
    /// error message and exit.
    RestartExec,
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

struct ClientInner {
    plugins: sync::RwLock<PluginRegister>,
    config: config::ClientConfiguration,
    state: sync::RwLock<ClientState>,
}

#[derive(Clone)]
pub struct Client(sync::Arc<ClientInner>);

impl Client {
    pub fn new(plugins: PluginRegister, config: config::ClientConfiguration) -> Client {
        let state = sync::RwLock::new(ClientState::new(config.nick.clone()));
        let inner = ClientInner {
            plugins: sync::RwLock::new(plugins),
            config: config,
            state: state,
        };
        return Client(sync::Arc::new(inner));
    }

    pub fn plugins(&self) -> &sync::RwLock<PluginRegister> {
        return &self.0.plugins;
    }

    pub fn config(&self) -> &config::ClientConfiguration {
        return &self.0.config;
    }

    pub fn state(&self) -> &sync::RwLock<ClientState> {
        return &self.0.state;
    }
}

impl irc::HasNick for Client {
    fn with_current_nick<T, F>(&self, fun: F) -> T
            where F: Fn(&str) -> T {
        fun(&self.0.state.read().unwrap().nick)
    }
}

/// This allows access to configuration fields directly on Client
impl ops::Deref for Client {
    type Target = config::ClientConfiguration;

    fn deref(&self) -> &config::ClientConfiguration {
        return self.config();
    }
}
