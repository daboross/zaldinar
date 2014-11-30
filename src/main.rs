extern crate irc;

mod plugins {
    pub mod control;
}

fn main() {
    let config = match irc::config::load_config_from_file(&Path::new("config.json")) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            std::os::set_exit_status(1);
            return
        }
    };

    let mut client = match irc::Client::new(config) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            std::os::set_exit_status(1);
            return
        }
    };

    plugins::control::register(&mut client);

    match client.connect() {
        Ok(()) => (),
        Err(e) => {
            println!("Error connecting: {}", e);
            std::os::set_exit_status(1);
            // There is no need to stop other tasks at this point, because the only time client.connect() returns Err is before any tasks are started
            return
        }
    }
}
