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
            where T: Fn(&events::MessageEvent) + Send + Sync + 'static {
        let boxed = sync::Arc::new(box f as MessageListener);
        let command_string = irc_command.to_string().to_ascii_lowercase();

        match self.raw_listeners.entry(command_string) {
            hash_map::Entry::Occupied(mut e) => e.get_mut().push(boxed),
            hash_map::Entry::Vacant(e) => drop(e.insert(vec!(boxed))),
        }
    }

    pub fn register_ctcp<T>(&mut self, ctcp_command: &str, f: T)
            where T: Fn(&events::CtcpEvent) + Send + Sync + 'static {
        let boxed = sync::Arc::new(box f as CtcpListener);
        let command_string = ctcp_command.to_string().to_ascii_lowercase();

        match self.ctcp_listeners.entry(command_string) {
            hash_map::Entry::Occupied(mut e) => e.get_mut().push(boxed),
            hash_map::Entry::Vacant(e) => drop(e.insert(vec!(boxed))),
        }
    }


    pub fn register_catch_all<T>(&mut self, f: T)
            where T: Fn(&events::MessageEvent) + Send + Sync + 'static {
        self.catch_all.push(sync::Arc::new(box f as MessageListener));
    }

    pub fn register_command<T>(&mut self, command: &str, f: T)
            where T: Fn(&events::CommandEvent) + Send + Sync + 'static {
        let boxed = sync::Arc::new(box f as CommandListener);
        let command_lower = command.to_string().to_ascii_lowercase();

        match self.commands.entry(command_lower) {
            hash_map::Entry::Occupied(mut e) => e.get_mut().push(boxed),
            hash_map::Entry::Vacant(e) => drop(e.insert(vec!(boxed))),
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

impl irc::HasNick for sync::Arc<Client> {
    fn with_current_nick<T, F>(&self, fun: F) -> T
            where F: Fn(&str) -> T {
        fun(&self.state.read().unwrap().nick)
    }
}

/// This allows access to configuration fields directly on Client
impl ops::Deref for Client {
    type Target = config::ClientConfiguration;

    fn deref<'a>(&'a self) -> &'a config::ClientConfiguration {
        return &self.config;
    }
}
