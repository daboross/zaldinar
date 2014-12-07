/// File containing responses for CTCP messages
extern crate chrono;

use self::chrono::Local;

use irc;
use irc::Client;
use irc::CtcpEvent;

fn ctcp_version(event: &CtcpEvent) {
    if !event.mask.has_nick() {
        return; // CTCP must come from a user
    }
    let message = format!("zaldinar - by Dabo - https://github.com/daboross/zaldinar - version {}", irc::get_version());
    event.client.send_ctcp_reply(event.mask.nick().unwrap(), event.command, message.as_slice());
}

fn ctcp_ping(event: &CtcpEvent) {
    if !event.mask.has_nick() {
        return; // CTCP must come from a user
    }
    // just send back the exact same message as a notice
    event.client.send_ctcp_reply(event.mask.nick().unwrap(), event.command, event.content);
}

fn ctcp_time(event: &CtcpEvent) {
    if !event.mask.has_nick() {
        return; // CTCP must come from a user
    }
    let message = format!("Current time: {}", Local::now().to_string());
    event.client.send_ctcp_reply(event.mask.nick().unwrap(), event.command, message.as_slice());
}

pub fn register(client: &mut Client) {
    client.add_ctcp_listener("version", ctcp_version);
    client.add_ctcp_listener("ping", ctcp_ping);
    client.add_ctcp_listener("time", ctcp_time);
}
