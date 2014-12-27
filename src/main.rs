#![feature(phase)]
extern crate zaldinar;
extern crate getopts;

use std::os;
use std::io;

fn print_err(msg: String) {
    if let Err(e) = writeln!(&mut io::stderr(), "{}", msg) {
        panic!("Failed to write to stderr.\nOriginal error output: {}\nSecondary error writing to stderr: {}", msg, e);
    }
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
            print_err(e.to_string());
            return;
        }
    };

    if matches.opt_present("help") {
        println!("{}", getopts::usage(getopts::short_usage(program, opts).as_slice(), opts));
        return;
    } else if matches.opt_present("version") {
        println!("zaldinar version {}", zaldinar::get_version());
        return;
    }

    let config_path = match matches.opt_str("config") {
        Some(v) => {
            let absolute = match os::make_absolute(&Path::new(v)) {
                Ok(v) => v,
                Err(e) => {
                    print_err(format!("Failed to make path absolute: {}", e));
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
                print_err(format!("Error loading configuration: {}", e));
                std::os::set_exit_status(1);
                return;
            },
        };

        match zaldinar::client::run(config) {
            Ok(true) => break,
            Ok(false) => println!("Restarting."),
            Err(e) => {
                println!("Error running client: {}", e);
                std::os::set_exit_status(1);
                // There is no need to stop other tasks at this point, because the only time client.connect() returns Err is before any tasks are started
            },
        };
    }
}
