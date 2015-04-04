use std::borrow::Borrow;
use std::sync;
use std::sync::mpsc;
use std::ops;

use regex;

use errors::InitializationError;
use client;
use events;
use irc;

#[derive(Clone)]
pub struct IrcInterface {
    data_out: mpsc::Sender<Option<String>>,
    pub client: sync::Arc<client::Client>,
    admins: sync::Arc<Vec<regex::Regex>>,
}

impl IrcInterface {
    pub fn new(data_out: mpsc::Sender<Option<String>>, client: sync::Arc<client::Client>)
            -> Result<IrcInterface, InitializationError> {
        let mut admins = Vec::new();
        for admin_str in client.config.admins.iter() {
            admins.push(try!(regex::Regex::new(&format!("^{}$", &admin_str))));
        }
        let interface = IrcInterface {
            data_out: data_out,
            client: client,
            admins: sync::Arc::new(admins),
        };
        return Ok(interface);
    }

    pub fn send_raw(&self, line: String) {
        if let Err(_) = self.data_out.send(Some(line)) {
            warn!("Unable to send to data_out from IrcInterface.");
        }
    }

    // TODO: replace CT: Borrow<str> with IntoCow or Into<Cow> when one of those becomes stable
    pub fn send_command<'_, CT, I>(&self, command: CT, args: &[I]) where
            CT: Borrow<str>, I: Borrow<str>, {
        let mut line: String = command.borrow().to_string();
        for item in args {
            line.push(' ');
            line.push_str(item.borrow());
        }
        self.send_raw(line);
    }

    pub fn send_message<T1: Borrow<str>, T2: Borrow<str>>(&self, target: T1, message: T2) {
        let line = format!("PRIVMSG {} :{}", target.borrow(), message.borrow());
        self.send_raw(line);
    }

    pub fn send_notice<T1: Borrow<str>, T2: Borrow<str>>(&self, target: T1, message: T2) {
        let line = format!("NOTICE {} :{}", target.borrow(), message.borrow());
        self.send_raw(line);
    }

    pub fn reply_notice<T: Borrow<str>>(&self, event: &events::CommandEvent, message: T) {
        if let Some(nick) = event.mask().nick() {
            let line = format!("NOTICE {} :{}", nick, message.borrow());
            self.send_raw(line);
        }
    }

    pub fn send_ctcp<T1, T2, T3>(&self, target: T1, command: T2, message: T3)
            where T1: Borrow<str>, T2: Borrow<str>, T3: Borrow<str> {

        let line = format!("PRIVMSG {} :\x01{} {}\x01", target.borrow(), command.borrow(),
            message.borrow());
        self.send_raw(line);
    }

    pub fn send_ctcp_reply<T1, T2, T3>(&self, target: T1, command: T2, content: T3)
            where T1: Borrow<str>, T2: Borrow<str>, T3: Borrow<str> {

        let line = format!("NOTICE {} :\x01{} {}\x01",
            target.borrow(), command.borrow(), content.borrow());
        self.send_raw(line);
    }

    pub fn join<T: Borrow<str>>(&self, channel: T) {
        let line = format!("JOIN :{}", channel.borrow());
        self.send_raw(line);
    }

    pub fn part<T1: Borrow<str>, T2: Borrow<str>>(&self, channel: T1, message: Option<T2>) {
        let line = match message {
            Some(m) => format!("PART {} :{}", channel.borrow(), m.borrow()),
            None => format!("PART {}", channel.borrow()),
        };
        self.send_raw(line);
    }

    pub fn quit<T: Borrow<str>>(&self, message: Option<T>, restart: client::ExecutingState) {
        let line = match message {
            Some(m) => format!("QUIT :{}", m.borrow()),
            None => format!("QUIT"),
        };
        {
            let mut state = self.client.state.write().unwrap();
            state.done_executing = restart;
        }
        self.send_raw(line);
        if let Err(_) =  self.data_out.send(None) {
            warn!("Unable to send to data_out from IrcInterface. (running quit)");
        }
    }

    pub fn is_admin(&self, event: &events::CommandEvent) -> bool {
        if self.is_mask_admin(event.mask()) {
            return true;
        } else {
            self.send_message(event.channel(), "Permission denied");
            return false;
        }
    }

    pub fn is_mask_admin(&self, mask: &events::IrcMask) -> bool {
        if let Some(mask_internal) = mask.mask() {
            if self.admins.iter().any(|r| r.is_match(mask_internal)) {
                return true;
            }
        }
        return false;
    }

    pub fn is_internal_mask_admin(&self, mask: &irc::IrcMask) -> bool {
        if let Some(mask_internal) = mask.mask() {
            if self.admins.iter().any(|r| r.is_match(mask_internal)) {
                return true;
            }
        }
        return false;
    }
}

/// This allows access to client and config fields on IrcInterface.
impl ops::Deref for IrcInterface {
    type Target = sync::Arc<client::Client>;

    fn deref<'a>(&'a self) -> &'a sync::Arc<client::Client> {
        return &self.client;
    }
}
