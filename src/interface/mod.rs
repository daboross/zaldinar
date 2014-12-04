use std::sync::Arc;
use regex::Regex;

use config::ClientConfiguration;
use errors::InitializationError;

pub struct IrcInterface {
    data_out: Sender<Option<String>>,
    pub config: Arc<ClientConfiguration>,
    admins: Arc<Vec<Regex>>,
}

impl IrcInterface {
    pub fn new(data_out: Sender<Option<String>>, config: Arc<ClientConfiguration>) -> Result<IrcInterface, InitializationError> {
        let mut admins: Vec<Regex> = Vec::new();
        for admin_str in config.admins.iter() {
            admins.push(try!(Regex::new(format!("^{}$", admin_str.as_slice()).as_slice())));
        }
        let interface = IrcInterface {
            data_out: data_out,
            config: config,
            admins: Arc::new(admins),
        };
        return Ok(interface);
    }

    pub fn send_raw(&self, line: String) {
        self.data_out.send(Some(line));
    }

    pub fn send_command<'a>(&self, command: String, args: &[&str]) {
        let mut line = command;
        line.push(' ');
        line.push_str(args.connect(" ").as_slice());
        self.send_raw(line);
    }

    pub fn send_message(&self, target: &str, message: &str) {
        let line = format!("PRIVMSG {} :{}", target, message);
        self.send_raw(line);
    }

    pub fn join(&self, channel: &str) {
        let line = format!("JOIN :{}", channel);
        self.send_raw(line);
    }

    pub fn part(&self, channel: &str, message: Option<&str>) {
        let line = match message {
            Some(m) => format!("PART {} :{}", channel, m),
            None => format!("PART {}", channel),
        };
        self.send_raw(line);
    }

    pub fn quit(&self, message: Option<&str>) {
        let line = match message {
            Some(m) => format!("QUIT :{}", m),
            None => format!("QUIT"),
        };
        self.send_raw(line);
        self.data_out.send(None);
    }

    pub fn is_admin(&self, event: &CommandEvent) -> bool {
        if event.mask.is_some() {
            let mask = event.mask.unwrap().as_slice();
            if self.admins.iter().any(|r| r.is_match(mask)) {
                return true;
            }
        }
        self.send_message(event.channel, "Permission denied");
        return false;
    }

}

impl Clone for IrcInterface {
    fn clone(&self) -> IrcInterface {
        return IrcInterface {
            data_out: self.data_out.clone(),
            config: self.config.clone(),
            admins: self.admins.clone(),
        };
    }
}


pub struct IrcMessageEvent<'a> {
    pub client: &'a IrcInterface,
    pub command: &'a str,
    pub args: &'a [&'a str],
    pub mask: Option<&'a str>,
    /// (ctcp_command, ctcp_message)
    pub ctcp: Option<(&'a str, &'a str)>
}

pub struct CommandEvent<'a> {
    pub client: &'a IrcInterface,
    pub channel: &'a str,
    pub args: &'a [&'a str],
    pub mask: Option<&'a str>,
}


impl <'a> IrcMessageEvent<'a> {
    pub fn new(client: &'a IrcInterface, command: &'a str, args: &'a [&'a str], mask: Option<&'a str>, ctcp: Option<(&'a str, &'a str)>) -> IrcMessageEvent<'a> {
        return IrcMessageEvent {
            client: client,
            command: command,
            args: args,
            mask: mask,
            ctcp: ctcp,
        };
    }
}

impl <'a> CommandEvent<'a> {
    pub fn new(client: &'a IrcInterface, channel: &'a str, args: &'a [&'a str], mask: Option<&'a str>) -> CommandEvent<'a> {
        return CommandEvent {
            client: client,
            channel: channel,
            args: args,
            mask: mask,
        };
    }
}

