#![allow(dead_code)]
use std::ops;

use irc;
use interface::IrcInterface;

#[derive(Clone)]
pub struct FullIrcMask {
    mask: String,
    nick: String,
    user: String,
    host: String,
}

impl FullIrcMask {
    pub fn mask(&self) -> &str {
        self.mask.as_slice()
    }

    pub fn nick(&self) -> &str {
        self.nick.as_slice()
    }

    pub fn user(&self) -> &str {
        self.user.as_slice()
    }

    pub fn host(&self) -> &str {
        self.host.as_slice()
    }
}

#[derive(Clone)]
pub enum IrcMask {
    Full(FullIrcMask),
    Unparseable(String),
    Nonexistent,
}

impl IrcMask {
    pub fn from_internal(mask: &irc::IrcMask) -> IrcMask {
        return match mask {
            &irc::IrcMask::Full(ref full_mask) => {
                IrcMask::Full(FullIrcMask {
                    mask: full_mask.mask.clone(),
                    nick: full_mask.nick.clone(),
                    user: full_mask.user.clone(),
                    host: full_mask.host.clone(),
                })
            },
            &irc::IrcMask::Unparseable(ref unparseable_mask) => {
                IrcMask::Unparseable(unparseable_mask.clone())
            },
            &irc::IrcMask::Nonexistent => {
                IrcMask::Nonexistent
            },
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
            &IrcMask::Full(ref m) => Some(m.mask.as_slice()),
            &IrcMask::Unparseable(ref m) => Some(m.as_slice()),
            &IrcMask::Nonexistent => None
        }
    }

    pub fn nick(&self) -> Option<&str> {
        match self {
            &IrcMask::Full(ref m) => Some(m.nick.as_slice()),
            &IrcMask::Unparseable(_) => None,
            &IrcMask::Nonexistent => None
        }
    }
}

pub struct MessageEvent<'a> {
    pub client: &'a IrcInterface,
    internal: &'a MessageTransport,
}

impl <'a> MessageEvent<'a> {
    pub fn new(client: &'a IrcInterface, internal: &'a MessageTransport) -> MessageEvent<'a> {
        return MessageEvent {
            client: client,
            internal: internal,
        }
    }
}

impl <'a> ops::Deref for MessageEvent<'a> {
    type Target = MessageTransport;

    fn deref(&self) -> &MessageTransport {
        self.internal
    }
}

pub struct CommandEvent<'a> {
    pub client: &'a IrcInterface,
    internal: &'a CommandTransport,
}

impl <'a> CommandEvent<'a> {
    pub fn new(client: &'a IrcInterface, internal: &'a CommandTransport) -> CommandEvent<'a> {
        return CommandEvent {
            client: client,
            internal: internal,
        }
    }
}

impl <'a> ops::Deref for CommandEvent<'a> {
    type Target = CommandTransport;

    fn deref(&self) -> &CommandTransport {
        self.internal
    }
}

pub struct CtcpEvent<'a> {
    pub client: &'a IrcInterface,
    internal: &'a CtcpTransport,
}

impl <'a> CtcpEvent<'a> {
    pub fn new(client: &'a IrcInterface, internal: &'a CtcpTransport) -> CtcpEvent<'a> {
        return CtcpEvent {
            client: client,
            internal: internal,
        }
    }
}

impl <'a> ops::Deref for CtcpEvent<'a> {
    type Target = CtcpTransport;

    fn deref(&self) -> &CtcpTransport {
        self.internal
    }
}

#[derive(Clone)]
pub struct MessageTransport {
    pub command: String,
    pub args: Vec<String>,
    pub mask: IrcMask,
    /// (ctcp_command, ctcp_message)
    pub ctcp: Option<(String, String)>,
    pub channel: Option<String>,
}

impl MessageTransport {
    pub fn from_internal(m: &irc::IrcMessage) -> MessageTransport {
        return MessageTransport {
            command: m.command.clone(),
            args: m.args.clone(),
            mask: IrcMask::from_internal(&m.mask),
            ctcp: m.ctcp.clone(),
            channel: m.channel.clone(),
        };
    }

    pub fn command(&self) -> &str {
        self.command.as_slice()
    }

    pub fn args(&self) -> &[String] {
        self.args.as_slice()
    }

    pub fn mask(&self) -> &IrcMask {
        &self.mask
    }

    pub fn ctcp(&self) -> Option<(&str, &str)> {
        self.ctcp.as_ref().map(|t| (t.0.as_slice(), t.1.as_slice()))
    }

    pub fn channel(&self) -> Option<&str> {
        self.channel.as_ref().map(|s| s.as_slice())
    }
}

#[derive(Clone)]
pub struct CommandTransport {
    pub channel: String,
    pub args: Vec<String>,
    pub mask: IrcMask,
}

impl CommandTransport {
    pub fn new(channel: &str, args: Vec<String>, mask: &irc::IrcMask) -> CommandTransport {
        return CommandTransport {
            channel: channel.to_string(),
            args: args,
            mask: IrcMask::from_internal(mask),
        }
    }

    pub fn mask(&self) -> &IrcMask {
        &self.mask
    }

    pub fn channel(&self) -> &str {
        self.channel.as_slice()
    }

    pub fn args(&self) -> &[String] {
        self.args.as_slice()
    }
}

#[derive(Clone)]
pub struct CtcpTransport {
    pub channel: String,
    pub command: String,
    pub content: String,
    pub mask: IrcMask,
}

impl CtcpTransport {
    pub fn from_internal(m: &irc::IrcMessage) -> Option<CtcpTransport> {
        return match m.ctcp {
            Some(ref tuple) => {
                Some(CtcpTransport {
                    channel: m.channel.as_ref().unwrap().clone(),
                    command: tuple.0.clone(),
                    content: tuple.1.clone(),
                    mask: IrcMask::from_internal(&m.mask),
                })
            },
            None => None,
        };
    }

    pub fn channel(&self) -> &str {
        self.channel.as_slice()
    }

    pub fn command(&self) -> &str {
        self.command.as_slice()
    }

    pub fn content(&self) -> &str {
        self.content.as_slice()
    }

    pub fn mask(&self) -> &IrcMask {
        &self.mask
    }
}
