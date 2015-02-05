use rand;
use rand::Rng;
use client::PluginRegister;
use events::CommandEvent;

fn choose(event: &CommandEvent) {
    let content = event.args.connect(" ");
    let mut rng = rand::thread_rng();
    let split = if content.contains(",") {
        regex!(r"\s*,\s*").split(&content).collect::<Vec<&str>>()
    } else {
        regex!(r"\s+").split(&content).collect::<Vec<&str>>()
    };
    let message = match rng.choose(&split) {
        Some(v) => *v,
        None => "I don't have anything to choose from.",
    };
    event.client.send_message(event.channel(), message);
}

fn coin(event: &CommandEvent) {
    let mut rng = rand::thread_rng();
    let message = format!("\x01ACTION flips a coin... \x02{}\x02\x01", rng.choose(&["heads",
                                                                            "tails"]).unwrap());
    event.client.send_message(event.channel(), &message);
}

fn rand(event: &CommandEvent) {
    if event.args.len() != 1 {
        event.client.send_message(event.channel(), "Please specify exactly one argument.");
        return;
    }
    let max = match event.args[0].parse::<u64>() {
        Ok(v) => v,
        Err(_) => {
            event.client.send_message(event.channel(),
                                        &format!("Invalid number '{}'", event.args[0]));
            return;
        },
    };
    let mut rng = rand::thread_rng();
    event.client.send_message(event.channel(), &format!("{}", rng.gen_range(0, max) + 1));
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("choose", choose);
    register.register_command("coin", coin);
    register.register_command("rand", rand);
}
