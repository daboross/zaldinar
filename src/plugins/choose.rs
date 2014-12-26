use std::rand;
use std::rand::Rng;
use client::PluginRegister;
use events::CommandEvent;

fn choose(event: &CommandEvent) {
    let content = event.args.connect(" ");
    let mut rng = rand::task_rng();
    let split = if content.contains(",") {
        regex!(r"\s*,\s*").split(content.as_slice()).collect::<Vec<&str>>()
    } else {
        regex!(r"\s+").split(content.as_slice()).collect::<Vec<&str>>()
    };
    let message = match rng.choose(split.as_slice()) {
        Some(v) => *v,
        None => "I don't have anything to choose from.",
    };
    event.client.send_message(event.channel(), message);
}

fn coin(event: &CommandEvent) {
    let mut rng = rand::task_rng();
    let message = format!("\x01ACTION flips a coin... \x02{}\x02\x01", rng.choose(&["heads", "tails"]).unwrap());
    event.client.send_message(event.channel(), message.as_slice());
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("choose", choose);
    register.register_command("coin", coin);
}
