extern crate "zaldinar" as irc;

mod plugins {
    pub mod control;
    pub mod log;
    pub mod ctcp;
}

fn main() {
    let config = match irc::ClientConfiguration::load_from_file(&Path::new("config.json")) {
        Ok(v) => v,
        Err(e) => {
            println!("Error loading configuration: {}", e);
            std::os::set_exit_status(1);
            return;
        },
    };

    let mut client = match irc::Client::new(config) {
        Ok(v) => v,
        Err(e) => {
            println!("Error initializing Client: {}", e);
            std::os::set_exit_status(1);
            return;
        },
    };

    plugins::control::register(&mut client);
    plugins::log::register(&mut client);
    plugins::ctcp::register(&mut client);

    match client.connect() {
        Ok(()) => (),
        Err(e) => {
            println!("Error connecting: {}", e);
            std::os::set_exit_status(1);
            // There is no need to stop other tasks at this point, because the only time client.connect() returns Err is before any tasks are started
            return;
        },
    }
}
