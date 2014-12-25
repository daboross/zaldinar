use std::ascii::AsciiExt;
use std::sync;
use std::thread;

use interface;
use irc;
use client;
use events;
use fern;
use fern_macros;

pub struct Dispatch {
    interface: interface::IrcInterface,
    state: sync::Arc<client::Client>,
    data_in: Receiver<Option<irc::IrcMessage>>,
    workers_out: Sender<SendPlugin>,
    workers_in: sync::Arc<sync::Mutex<Receiver<SendPlugin>>>,
}

impl Dispatch {
    pub fn new(interface: interface::IrcInterface, state: sync::Arc<client::Client>, data_in: Receiver<Option<irc::IrcMessage>>) -> Dispatch {
        let (worker_out, worker_in) = channel();

        return Dispatch {
            interface: interface,
            state: state,
            data_in: data_in,
            workers_out: worker_out,
            workers_in: sync::Arc::new(sync::Mutex::new(worker_in)),
        };
    }

    pub fn start_workers(&self, logger: fern::ArcLogger) {
        for i in range::<u8>(0, 4) {
            let executor = PluginExecutor::new(self.interface.clone(), self.workers_in.clone());
            executor.start_worker_thread(logger.clone(), format!("plugin_executor_{}", i));
        }
    }

    pub fn start_dispatch_loop(self) {
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

        // PING
        if message.command.as_slice().eq_ignore_ascii_case("PING") {
            self.interface.send_raw(format!("PONG {}", message.args.connect(" ")));
        }

        let message_event = events::MessageTransport::from_internal(message);

        // Catch all listeners
        for listener in plugins.catch_all.iter() {
            self.workers_out.send(SendPlugin::Message((listener.clone(), message_event.clone())));
        }

        // Raw listeners
        if let Some(list) = plugins.raw_listeners.get(&message.command.to_ascii_lower()) {
            for listener in list.iter() {
                self.workers_out.send(SendPlugin::Message((listener.clone(), message_event.clone())));
            }
        }

        if message.command.as_slice().eq_ignore_ascii_case("PRIVMSG") {
            let channel = message.channel.as_ref().unwrap().as_slice(); // Always exists for PRIVMSG

            // CTCP
            if let Some(ctcp_event) = events::CtcpTransport::from_internal(message) {
                if let Some(list) = plugins.ctcp_listeners.get(&message.args[0].to_ascii_lower()) {
                    for ctcp_listener in list.iter() {
                        self.workers_out.send(SendPlugin::Ctcp((ctcp_listener.clone(), ctcp_event.clone())));
                    }
                }
            }

            // Commands
            let command_prefix = format!(":{}", self.state.command_prefix.as_slice());

            // This checks for the command prefix, commands typed like '.command_name args'
            if message.args[1].starts_with(command_prefix.as_slice()) {
                let command = message.args[1].slice_from(command_prefix.len());
                let args = message.args.slice_from(2).iter().map(|s| s.clone()).collect::<Vec<String>>();
                self.dispatch_command(&plugins, command, channel, args, &message.mask);
            } else {
                // This checks for someone typing commands like 'BotName, command_name args'
                // We store whether or not a command was matched in a variable so that we can use it below.
                let mut command_matched = false;
                if let Some(captures) = regex!(r"^:([^\s]+?)[:;,]?\s+(.+)$").captures(message.args.slice_from(1).connect(" ").as_slice()) {
                    if captures.at(1) == Some(self.state.state.read().nick.as_slice()) {
                        if let Some(args_str) = captures.at(2) {
                            let split = args_str.split(' ').collect::<Vec<&str>>();
                            let command = split[0];
                            let args = split.slice_from(1).iter().map(|s| s.to_string()).collect::<Vec<String>>();
                            self.dispatch_command(&plugins, command, channel, args, &message.mask);
                            command_matched = true;
                        }
                    }
                }

                // This checks for commands in a private message, where a prefix isn't required
                // People can just say 'command args' in a private message.
                // If the channel is the sender's nick, the message is being sent in a private message.
                if !command_matched && message.mask.nick() == Some(channel) {
                    let command = message.args[1].slice_from(1); // slice_from(1) to remove the `:` at the beginning of privmsg content.
                    let args = message.args.slice_from(2).iter().map(|s| s.clone()).collect::<Vec<String>>();
                    self.dispatch_command(&plugins, command, channel, args, &message.mask);
                }
            }
        }
    }

    fn dispatch_command(&self, plugins: &sync::RWLockReadGuard<client::PluginRegister>, command: &str, channel: &str, args: Vec<String>, mask: &irc::IrcMask) {
        if let Some(list) = plugins.commands.get(&command.to_ascii_lower()) {
            let command_event = events::CommandTransport::new(channel, args, mask);
            for closure in list.iter() {
                self.workers_out.send(SendPlugin::Command((closure.clone(), command_event.clone())));
            }
        }
    }
}

/// TODO: Better name for this
enum SendPlugin {
    Command((sync::Arc<client::CommandListener>, events::CommandTransport)),
    Message((sync::Arc<client::MessageListener>, events::MessageTransport)),
    Ctcp((sync::Arc<client::CtcpListener>, events::CtcpTransport)),
}

impl SendPlugin {
    fn execute(self, interface: &interface::IrcInterface) {
        match self {
            SendPlugin::Command((closure, event)) => {
                closure.call((&events::CommandEvent::new(interface, &event),));
            },
            SendPlugin::Message((closure, event)) => {
                closure.call((&events::MessageEvent::new(interface, &event),));
            },
            SendPlugin::Ctcp((closure, event)) => {
                closure.call((&events::CtcpEvent::new(interface, &event),));
            },
        }
    }
}


struct PluginExecutor {
    interface: interface::IrcInterface,
    data_in: sync::Arc<sync::Mutex<Receiver<SendPlugin>>>,
}

impl PluginExecutor {
    fn new(interface: interface::IrcInterface, data_in: sync::Arc<sync::Mutex<Receiver<SendPlugin>>>) -> PluginExecutor {
        return PluginExecutor {
            interface: interface,
            data_in: data_in,
        };
    }

    fn worker_loop(&self) {
        loop {
            let next = self.data_in.lock().recv();
            next.execute(&self.interface);
        }
    }

    fn start_worker_thread(self, logger: fern::ArcLogger, name: String) {
        thread::Builder::new().name(name).spawn(move || {
            fern_macros::init_thread_logger(logger);
            self.worker_loop();
        }).detach();
    }
}
