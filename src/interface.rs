use std::sync;
use regex;

use errors::InitializationError;
use client;
use irc;

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

    pub fn is_admin(&self, event: &CommandEvent) -> bool {
        if event.mask.has_mask() {
            let mask = event.mask.mask().unwrap().as_slice();
            if self.admins.iter().any(|r| r.is_match(mask)) {
                return true;
            }
        }
        self.send_message(event.channel, "Permission denied");
        return false;
    }
}

/// This allows access to client and config fields on IrcInterface.
impl Deref<sync::Arc<client::Client>> for IrcInterface {
    fn deref<'a>(&'a self) -> &'a sync::Arc<client::Client> {
        return &self.client;
    }
}

pub enum IrcMask<'a> {
    Full(FullIrcMask<'a>),
    Unparseable(&'a str),
    Nonexistent,
}

pub struct FullIrcMask<'a> {
    pub mask: &'a str,
    pub nick: &'a str,
    pub user: &'a str,
    pub host: &'a str,
}

impl <'a> IrcMask<'a> {
    pub fn from_internal(mask: &'a irc::IrcMask) -> IrcMask<'a> {
        return match mask {
            &irc::IrcMask::Full(ref full_mask) => {
                IrcMask::Full(FullIrcMask {
                    mask: full_mask.mask.as_slice(),
                    nick: full_mask.nick.as_slice(),
                    user: full_mask.user.as_slice(),
                    host: full_mask.host.as_slice(),
                })
            },
            &irc::IrcMask::Unparseable(ref unparseable_mask) => {
                IrcMask::Unparseable(unparseable_mask.as_slice())
            },
            &irc::IrcMask::Nonexistent => {
                IrcMask::Nonexistent
            }
        };
    }

    pub fn has_mask(&self) -> bool {
        match self {
            &IrcMask::Full(_) => true,
            &IrcMask::Unparseable(_) => true,
            &IrcMask::Nonexistent => false,
        }
    }

    pub fn has_nick(&self) -> bool {
        match self {
            &IrcMask::Full(_) => true,
            &IrcMask::Unparseable(_) => false,
            &IrcMask::Nonexistent => false,
        }
    }

    pub fn mask(&self) -> Option<&str> {
        match self {
            &IrcMask::Full(ref m) => Some(m.mask),
            &IrcMask::Unparseable(m) => Some(m),
            &IrcMask::Nonexistent => None
        }
    }

    pub fn nick(&self) -> Option<&str> {
        match self {
            &IrcMask::Full(ref m) => Some(m.nick),
            &IrcMask::Unparseable(_) => None,
            &IrcMask::Nonexistent => None
        }
    }

}

pub struct IrcMessageEvent<'a> {
    pub client: &'a IrcInterface,
    pub command: &'a str,
    pub args: &'a [&'a str],
    pub mask: &'a IrcMask<'a>,
    /// (ctcp_command, ctcp_message)
    pub ctcp: Option<(&'a str, &'a str)>,
    pub channel: Option<&'a str>,
}

pub struct CommandEvent<'a> {
    pub client: &'a IrcInterface,
    pub channel: &'a str,
    pub args: &'a [&'a str],
    pub mask: &'a IrcMask<'a>,
}

pub struct CtcpEvent<'a> {
    pub client: &'a IrcInterface,
    pub channel: &'a str,
    pub command: &'a str,
    pub content: &'a str,
    pub mask: &'a IrcMask<'a>,
}


impl <'a> IrcMessageEvent<'a> {
    pub fn new(client: &'a IrcInterface, command: &'a str, args: &'a [&'a str], mask: &'a IrcMask, ctcp: Option<(&'a str, &'a str)>, channel: Option<&'a str>) -> IrcMessageEvent<'a> {
        return IrcMessageEvent {
            client: client,
            command: command,
            args: args,
            mask: mask,
            ctcp: ctcp,
            channel: channel,
        };
    }
}

impl <'a> CtcpEvent<'a> {
    pub fn new(client: &'a IrcInterface, channel: &'a str, command: &'a str, content: &'a str, mask: &'a IrcMask) -> CtcpEvent<'a> {
        return CtcpEvent {
            client: client,
            channel: channel,
            command: command,
            content: content,
            mask: mask,
        };
    }
}

impl <'a> CommandEvent<'a> {
    pub fn new(client: &'a IrcInterface, channel: &'a str, args: &'a [&'a str], mask: &'a IrcMask) -> CommandEvent<'a> {
        return CommandEvent {
            client: client,
            channel: channel,
            args: args,
            mask: mask,
        };
    }
}
