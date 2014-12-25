use std::sync;
use regex;

use errors::InitializationError;
use client;
use events;

#[deriving(Clone)]
pub struct IrcInterface {
    data_out: Sender<Option<String>>,
    pub client: sync::Arc<client::Client>,
    admins: sync::Arc<Vec<regex::Regex>>,
}

impl IrcInterface {
    pub fn new(data_out: Sender<Option<String>>, client: sync::Arc<client::Client>) -> Result<IrcInterface, InitializationError> {
        let mut admins = Vec::new();
        for admin_str in client.config.admins.iter() {
            admins.push(try!(regex::Regex::new(format!("^{}$", admin_str.as_slice()).as_slice())));
        }
        let interface = IrcInterface {
            data_out: data_out,
            client: client,
            admins: sync::Arc::new(admins),
        };
        return Ok(interface);
    }

    pub fn send_raw(&self, line: String) {
        self.data_out.send(Some(line));
    }

    pub fn send_command(&self, command: String, args: &[&str]) {
        let mut line = command;
        line.push(' ');
        line.push_str(args.connect(" ").as_slice());
        self.send_raw(line);
    }

    pub fn send_message(&self, target: &str, message: &str) {
        let line = format!("PRIVMSG {} :{}", target, message);
        self.send_raw(line);
    }

    pub fn send_notice(&self, target: &str, message: &str) {
        let line = format!("NOTICE {} :{}", target, message);
        self.send_raw(line);
    }

    pub fn send_ctcp_reply(&self, target: &str, command: &str, content: &str) {
        let line = format!("NOTICE {} :\x01{} {}\x01", target, command, content);
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

    pub fn is_admin(&self, event: &events::CommandEvent) -> bool {
        if event.mask().has_mask() {
            let mask = event.mask().mask().unwrap().as_slice();
            if self.admins.iter().any(|r| r.is_match(mask)) {
                return true;
            }
        }
        self.send_message(event.channel(), "Permission denied");
        return false;
    }
}

/// This allows access to client and config fields on IrcInterface.
impl Deref<sync::Arc<client::Client>> for IrcInterface {
    fn deref<'a>(&'a self) -> &'a sync::Arc<client::Client> {
        return &self.client;
    }
}
