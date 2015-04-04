use time;

use client::PluginRegister;
use events::CtcpEvent;
use VERSION;

fn ctcp_version(event: &CtcpEvent) {
    if !event.mask.has_nick() {
        return; // CTCP must come from a user
    }
    let message = format!("zaldinar - by Dabo - https://github.com/daboross/zaldinar - version {}",
        VERSION);
    event.client.send_ctcp_reply(event.mask.nick().unwrap(), event.command(), message);
}

fn ctcp_ping(event: &CtcpEvent) {
    if !event.mask.has_nick() {
        return; // CTCP must come from a user
    }
    // just send back the exact same message as a notice
    event.client.send_ctcp_reply(event.mask.nick().unwrap(), event.command(), event.content());
}

fn ctcp_time(event: &CtcpEvent) {
    if !event.mask.has_nick() {
        return; // CTCP must come from a user
    }
    let message = format!("Current time: {}",
        time::now().strftime("%Y-%m-%d - %H:%M:%S").unwrap());
    event.client.send_ctcp_reply(event.mask.nick().unwrap(), event.command(), message);
}

pub fn register(register: &mut PluginRegister) {
    register.register_ctcp("version", ctcp_version);
    register.register_ctcp("ping", ctcp_ping);
    register.register_ctcp("time", ctcp_time);
}
