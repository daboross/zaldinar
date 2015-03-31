#![allow(dead_code)] // this is for the things generated in resources/foods/cakes.rs
//! depends: rand = "0.3.*"
extern crate rand;
extern crate zaldinar_core;

use rand::Rng;

use zaldinar_core::client::PluginRegister;
use zaldinar_core::events::CommandEvent;

// This file is taken periodically from: http://en.wikipedia.org/wiki/List_of_cakes
// Specifically, using http://en.wikipedia.org/w/index.php?action=raw&title=List_of_cakes, and
// then a ton of regex replacements to get to json, and then more replacements to get to rust
// source! TODO: an automated method to parse the page.
// This creates a Cake struct and CAKES static variable
include!("resources/foods/cakes.rs");

fn cake(event: &CommandEvent) {
    let mut rng = rand::thread_rng();
    let cake = match rng.choose(CAKES) {
        Some(cake) => cake,
        None => {
            event.client.send_message(event.channel(), "No cakes found!");
            return;
        },
    };
    let message = match cake.location {
        "unknown" => format!(r#"The "{}" cake: {}!"#, cake.name, cake.info),
        _ => format!(r#"The "{}" cake: {} - founded in {}!"#, cake.name, cake.info, cake.location),
    };
    event.client.send_message(event.channel(), message);
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("randomcake", cake);
}
