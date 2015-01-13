#![allow(unstable)]
extern crate zaldinar;
extern crate getopts;

use std::os;
use std::io::stdio;

macro_rules! print_err {
    ($($arg:tt)*) => (
        if let Err(e) = write!(&mut stdio::stderr_raw(), $($arg)*) {
            panic!("Failed to write to stderr.\
                \nOriginal error output: {}\
                \nSecondary error writing to stderr: {}", format!($($arg)*), e);
        }
    )
}

fn main() {
    let args = os::args();
    let program = args[0].as_slice();
    let opts = &[
        getopts::optopt("c", "config", "set config file name", "FILE"),
        getopts::optflag("h", "help", "print this help menu"),
        getopts::optflag("v", "version", "print program version"),
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(v) => v,
        Err(e) => {
            print_err!("{}", e.to_string());
            return;
        }
    };

    if matches.opt_present("help") {
        println!("{}", getopts::usage(getopts::short_usage(program, opts).as_slice(), opts));
        return;
    } else if matches.opt_present("version") {
        println!("zaldinar version {}", zaldinar::VERSION);
        return;
    }

    let config_path = match matches.opt_str("config") {
        Some(v) => {
            let absolute = match os::make_absolute(&Path::new(v)) {
                Ok(v) => v,
                Err(e) => {
                    print_err!("Failed to make path absolute: {}", e);
                    return;
                }
            };
            println!("Using configuration: {}", absolute.display());
            absolute
        },
        None => Path::new("config.json"),
    };

    loop {
        let config = match zaldinar::ClientConfiguration::load_from_file(&config_path) {
            Ok(v) => v,
            Err(e) => {
                print_err!("Error loading configuration: {}", e);
                std::os::set_exit_status(1);
                return;
            },
        };

        match zaldinar::client::run(config) {
            Ok(zaldinar::client::ExecutingState::Done) => {
                println!("Done, exiting.");
                break
            },
            Ok(zaldinar::client::ExecutingState::Running) => {
                println!("Restarting zaldinar main loop.");
                continue;
            },
            Ok(zaldinar::client::ExecutingState::Restart) => {
                println!("Restarting zaldinar using exec.");
                continue; // TODO: exec
            }
            Err(e) => {
                println!("Error running client: {}", e);
                std::os::set_exit_status(1);
                // There is no need to stop other tasks at this point, because the only time
                // client.connect() returns Err is before any tasks are started
            },
        };
    }
}
