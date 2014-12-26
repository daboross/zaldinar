use std::rand;
use std::rand::Rng;
use client::PluginRegister;
use events::CommandEvent;

fn eightball(event: &CommandEvent) {
    if event.args.is_empty() {
        event.client.send_message(event.channel(), "I can't answer if you don't ask.");
        return;
    }
    let messages = include_str!("../../resources/8ball/messages.txt").split('\n').collect::<Vec<&str>>();
    let mut rng = rand::task_rng();
    let message = rng.choose(messages.as_slice()).unwrap()
                    .replace("<yes>", "\x0305").replace("<no>", "\x0303");
    event.client.send_message(event.channel(), format!("\x01ACTION shakes the magic 8 ball... \x02{}\x01", message).as_slice());
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("8ball", eightball);
}
