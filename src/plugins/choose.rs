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

fn rand(event: &CommandEvent) {
    if event.args.len() != 1 {
        event.client.send_message(event.channel(), "Please specify exactly one argument.");
    }
    let max = match event.args[0].parse::<u64>() {
        Some(v) => v,
        None => {
            event.client.send_message(event.channel(), format!("Invalid number '{}'", event.args[0]).as_slice());
            return;
        },
    };
    let mut rng = rand::task_rng();
    event.client.send_message(event.channel(), format!("{}", rng.gen_range(1, max + 1)).as_slice())
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("choose", choose);
    register.register_command("coin", coin);
    register.register_command("rand", rand);
}
