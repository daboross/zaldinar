use std::ascii::AsciiExt;
use std::sync;

use interface;
use irc;
use client;

pub struct Dispatch {
    interface: interface::IrcInterface,
    state: sync::Arc<client::Client>,
    data_in: Receiver<Option<irc::IrcMessage>>,
}

impl Dispatch {
    pub fn new(interface: interface::IrcInterface, state: sync::Arc<client::Client>, data_in: Receiver<Option<irc::IrcMessage>>) -> Dispatch {
        return Dispatch {
            interface: interface,
            state: state,
            data_in: data_in,
        };
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

        let shared_mask = &interface::IrcMask::from_internal(&message.mask);
        let shared_args = message.args.iter().map(|s| s.as_slice()).collect::<Vec<&'a str>>();
        let shared_ctcp = message.ctcp.as_ref().map(|t| (t.0.as_slice(), t.1.as_slice()));
        let shared_channel = message.channel.as_ref().map(|s| s.as_slice());

        // PING
        if message.command.as_slice().eq_ignore_ascii_case("PING") {
            self.interface.send_command("PONG".to_string(), shared_args.as_slice());
        }

        let message_event = &mut interface::IrcMessageEvent::new(&self.interface, message.command.as_slice(), shared_args.as_slice(), shared_mask, shared_ctcp, shared_channel);

        // Catch all listeners
        for listener in plugins.catch_all.iter() {
            listener.call((message_event,));
        }

        // Raw listeners
        if let Some(list) = plugins.raw_listeners.get(&message.command.to_ascii_lower()) {
            for listener in list.iter() {
                listener.call((message_event,));
            }
        }

        if message.command.as_slice().eq_ignore_ascii_case("PRIVMSG") {
            let channel = shared_channel.unwrap(); // Always exists for PRIVMSG

            // CTCP
            if let Some(ref t) = message.ctcp {
                if let Some(list) = plugins.ctcp_listeners.get(&message.args[0].to_ascii_lower()) {
                    let ctcp_event = interface::CtcpEvent::new(&self.interface, message.args[0].as_slice(), t.0.as_slice(), t.1.as_slice(), shared_mask);
                    for ctcp_listener in list.iter() {
                        ctcp_listener.call((&ctcp_event,));
                    }
                }
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
                let mut command_matched = false;
                if let Some(captures) = regex!(r"^:([^\s]+?)[:;,]?\s+(.+)$").captures(shared_args.slice_from(1).connect(" ").as_slice()) {
                    if captures.at(1) == Some(self.state.state.read().nick.as_slice()) {
                        if let Some(args_str) = captures.at(2) {
                            let split = args_str.split(' ').collect::<Vec<&str>>();
                            let command = split[0];
                            let args = split.slice_from(1);
                            self.dispatch_command(&plugins, command, channel, args, shared_mask);
                            command_matched = true;
                        }
                    }
                }

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

    fn dispatch_command(&self, plugins: &sync::RWLockReadGuard<client::PluginRegister>, command: &str, channel: &str, args: &[&str], mask: &interface::IrcMask) {
        if let Some(list) = plugins.commands.get(&command.to_ascii_lower()) {
            let command_event = &mut interface::CommandEvent::new(&self.interface, channel, args, mask);
            for closure in list.iter() {
                closure.call((command_event,));
            }
        }
    }
}
