//! depends: rand = "0.3"
//! depends: regex = "0.2"

extern crate zaldinar_core;
extern crate regex;
extern crate rand;

use rand::Rng;

use zaldinar_core::client::PluginRegister;
use zaldinar_core::events::CommandEvent;

macro_rules! regex {
    ($s:expr) => (::regex::Regex::new($s).unwrap())
}

fn choose(event: &CommandEvent) {
    let content = event.args.join(" ");
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
    event.client.send_message(event.channel(), message);
}

fn rand_command(event: &CommandEvent) {
    if event.args.len() != 1 {
        event.client.send_message(event.channel(), "Please specify exactly one argument.");
        return;
    }
    let max = match event.args[0].parse::<u64>() {
        Ok(v) => v,
        Err(_) => {
            event.client.send_message(event.channel(),
                                        format!("Invalid number '{}'", event.args[0]));
            return;
        },
    };
    let mut rng = rand::thread_rng();
    event.client.send_message(event.channel(), format!("{}", rng.gen_range(0, max) + 1));
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("choose", choose);
    register.register_command("coin", coin);
    register.register_command("rand", rand_command);
}
