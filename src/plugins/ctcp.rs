use chrono::Local;

use client::PluginRegister;
use interface::CtcpEvent;
use get_version;

fn ctcp_version(event: &CtcpEvent) {
    if !event.mask.has_nick() {
        return; // CTCP must come from a user
    }
    let message = format!("zaldinar - by Dabo - https://github.com/daboross/zaldinar - version {}", get_version());
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

pub fn register(register: &mut PluginRegister) {
    register.register_ctcp("version", ctcp_version);
    register.register_ctcp("ping", ctcp_ping);
    register.register_ctcp("time", ctcp_time);
}
