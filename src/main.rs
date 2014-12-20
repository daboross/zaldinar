extern crate zaldinar;

fn main() {
    let config = match zaldinar::ClientConfiguration::load_from_file(&Path::new("config.json")) {
        Ok(v) => v,
        Err(e) => {
            println!("Error loading configuration: {}", e);
            std::os::set_exit_status(1);
            return;
        },
    };

    let client = match zaldinar::Client::new(config) {
        Ok(v) => v,
        Err(e) => {
            println!("Error initializing Client: {}", e);
            std::os::set_exit_status(1);
            return;
        },
    };

    match client.connect() {
        Ok(()) => (),
        Err(e) => {
            println!("Error connecting: {}", e);
            std::os::set_exit_status(1);
            // There is no need to stop other tasks at this point, because the only time client.connect() returns Err is before any tasks are started
        },
    }
}
