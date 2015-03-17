use std::ascii::AsciiExt;
use std::sync;
use std::sync::mpsc;
use std::thread;
use std::fmt;
use std::io;

use core::interface;
use core::client;
use core::events;
use irc;

pub struct Dispatch {
    interface: interface::IrcInterface,
    state: sync::Arc<client::Client>,
    data_in: mpsc::Receiver<irc::IrcMessage>,
    workers_out: mpsc::Sender<PluginThunk>,
}

impl Dispatch {
    pub fn new(interface: interface::IrcInterface, state: sync::Arc<client::Client>,
            data_in: mpsc::Receiver<irc::IrcMessage>) -> Dispatch {
        let (dispatch_out, workers_in) = mpsc::channel();
        let workers_in_arc = sync::Arc::new(sync::Mutex::new(workers_in));
        for _ in 0..4 {
            let executor = PluginExecutor::new(interface.clone(), workers_in_arc.clone());
            executor.start_worker_thread();
        }

        return Dispatch {
            interface: interface,
            state: state,
            data_in: data_in,
            workers_out: dispatch_out,
        };
    }

    pub fn dispatch_loop(self) {
        loop {
            let message = match self.data_in.recv() {
                Ok(v) => v,
                Err(_) => break,
            };
            if let Err(_) = self.process_message(&message) {
                error!("Failed to send to workers_out from Dispatch. Exiting.");
                return;
            }
        }
    }

    pub fn start_dispatch_loop(self) -> io::Result<()> {
        try!(thread::Builder::new().name("dispatch".to_string()).scoped(move || {
            self.dispatch_loop();
        })).join();
        return Ok(());
    }

    fn process_message<'a>(&self, message: &'a irc::IrcMessage)
            -> Result<(), mpsc::SendError<PluginThunk>> {
        let plugins = self.state.plugins.read().unwrap();

        // PING
        if message.command.eq_ignore_ascii_case("PING") {
            self.interface.send_raw(format!("PONG {}", message.args.connect(" ")));
        }

        let message_event = events::MessageTransport::from_internal(message);

        // Catch all listeners
        for listener in &plugins.catch_all {
            try!(self.execute(PluginThunk::Message((listener.clone(), message_event.clone()))));
        }

        // Raw listeners
        if let Some(list) = plugins.raw_listeners.get(&message.command.to_ascii_lowercase()) {
            for listener in list {
                try!(self.execute(PluginThunk::Message((listener.clone(), message_event.clone()))));
            }
        }

        if message.command.eq_ignore_ascii_case("PRIVMSG") {
            // Channel always exists for PRIVMSG
            let channel = &message.channel.as_ref().unwrap();

            // CTCP
            if let Some(ctcp_event) = events::CtcpTransport::from_internal(message) {
                if let Some(list) = plugins.ctcp_listeners.get(&ctcp_event.command
                        .to_ascii_lowercase()) {
                    for ctcp_listener in list {
                        try!(self.execute(PluginThunk::Ctcp((ctcp_listener.clone(),
                            ctcp_event.clone()))));
                    }
                }
            }

            // Commands
            let command_prefix = format!(":{}", &self.state.command_prefix);

            // This checks for the command prefix, commands typed like '.command_name args'
            if message.args[1].starts_with(&command_prefix) {
                let command = &message.args[1][command_prefix.len()..];
                let args = message.args[2..].iter().map(|s| s.clone())
                            .collect::<Vec<String>>();
                try!(self.dispatch_command(&plugins, command, channel, args, &message.mask));
            } else {
                // This checks for someone typing commands like 'BotName, command_name args'
                // We store whether or not a command was matched in a variable so that we can use
                // it below.
                let mut command_matched = false;
                if let Some(captures) = regex!(r"^:([^\s]+?)[:;,]?\s+(.+)$").captures(
                                            &message.args[1..].connect(" ")) {
                    let same = {
                        let state = self.state.state.read().unwrap();
                        captures.at(1) == Some(&state.nick)
                    };
                    if same {
                        if let Some(args_str) = captures.at(2) {
                            let split = args_str.split(' ').collect::<Vec<&str>>();
                            let command = split[0];
                            let args = split[1..].iter().map(|s| s.to_string())
                                        .collect::<Vec<String>>();
                            try!(self.dispatch_command(&plugins, command, channel, args, &message.mask));
                            command_matched = true;
                        }
                    }
                }

                // This checks for commands in a private message, where a prefix isn't required
                // People can just say 'command args' in a private message. If the channel is the
                // sender's nick, the message is being sent in a private message.
                if !command_matched && message.mask.nick() == Some(channel) {
                    // [1..] to remove the `:` at the beginning of privmsg content.
                    let command = &message.args[1][1..];
                    let args = message.args[2..].iter().map(|s| s.clone())
                                .collect::<Vec<String>>();
                    try!(self.dispatch_command(&plugins, command, channel, args, &message.mask));
                }
            }
        }
        return Ok(());
    }

    fn dispatch_command(&self, plugins: &sync::RwLockReadGuard<client::PluginRegister>,
            command: &str, channel: &str, args: Vec<String>, mask: &irc::IrcMask)
            -> Result<(), mpsc::SendError<PluginThunk>> {
        if let Some(list) = plugins.commands.get(&command.to_ascii_lowercase()) {
            let command_event = events::CommandTransport::new(channel, args, mask);
            for closure in list {
                try!(self.execute(PluginThunk::Command((closure.clone(), command_event.clone()))));
            }
        }
        return Ok(());
    }

    fn execute(&self, task: PluginThunk) -> Result<(), mpsc::SendError<PluginThunk>> {
        self.workers_out.send(task)
    }
}

/// TODO: Better name for this
enum PluginThunk {
    Command((sync::Arc<client::CommandListener>, events::CommandTransport)),
    Message((sync::Arc<client::MessageListener>, events::MessageTransport)),
    Ctcp((sync::Arc<client::CtcpListener>, events::CtcpTransport)),
}

impl PluginThunk {
    fn execute(self, interface: &interface::IrcInterface) {
        match self {
            PluginThunk::Command((closure, event)) => {
                (*closure)(&events::CommandEvent::new(interface, &event));
            },
            PluginThunk::Message((closure, event)) => {
                (*closure)(&events::MessageEvent::new(interface, &event));
            },
            PluginThunk::Ctcp((closure, event)) => {
                (*closure)(&events::CtcpEvent::new(interface, &event));
            },
        }
    }
}

impl fmt::Display for PluginThunk {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_str(match self {
            &PluginThunk::Command(_) => "command",
            &PluginThunk::Message(_) => "message",
            &PluginThunk::Ctcp(_) => "ctcp",
        })
    }
}


struct PluginExecutor {
    interface: interface::IrcInterface,
    data_in: sync::Arc<sync::Mutex<mpsc::Receiver<PluginThunk>>>,
    active: bool,
}

impl PluginExecutor {
    fn new(interface: interface::IrcInterface,
            data_in: sync::Arc<sync::Mutex<mpsc::Receiver<PluginThunk>>>)
            -> PluginExecutor {
        return PluginExecutor {
            interface: interface,
            data_in: data_in,
            active: true,
        };
    }

    fn worker_loop(&mut self) {
        loop {
            let message = {
                // Only lock data_in for the time it takes to get a job, not run it.
                let lock = self.data_in.lock().unwrap();
                lock.recv()
            };
            match message {
                Ok(next) => {
                    let desc = format!("{}", &next);
                    debug!("Executing {}", desc);
                    next.execute(&self.interface);
                    debug!("Done executing {}", desc);
                },
                Err(_) => {
                    self.active = false;
                    break;
                }
            };
        }
    }

    fn start_worker_thread(mut self) {
        let r = thread::Builder::new().name("worker_thread".to_string()).spawn(move ||
            self.worker_loop());
        if let Err(e) = r {
            error!("Failed to start new worker thread! Plugins will no longer have a full set of \
                workers to run on! Error: {}", e)
        }
    }
}

impl Drop for PluginExecutor {
    fn drop(&mut self) {
        if self.active {
            warn!("Worker panicked!");
            PluginExecutor::new(self.interface.clone(), self.data_in.clone(),)
                .start_worker_thread();
        }
    }
}
