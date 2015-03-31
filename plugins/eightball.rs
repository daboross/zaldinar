//! depends: rand = "0.2.*"
extern crate zaldinar_core;
extern crate rand;
use rand::Rng;
use zaldinar_core::client::PluginRegister;
use zaldinar_core::events::CommandEvent;

const MESSAGES: &'static str = include_str!("resources/eightball/responses.txt");

fn eightball(event: &CommandEvent) {
    if event.args.is_empty() {
        event.client.send_message(event.channel(), "I can't answer if you don't ask.");
        return;
    }
    let messages = MESSAGES.split('\n').collect::<Vec<&str>>();
    let mut rng = rand::thread_rng();
    let message = rng.choose(&messages).unwrap()
                    .replace("<yes>", "\x0305").replace("<no>", "\x0303");
    event.client.send_message(event.channel(), format!("\x01ACTION shakes the magic 8 ball... \x02{}\x01", message));
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("8ball", eightball);
}
