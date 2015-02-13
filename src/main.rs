#![feature(io, path, env, std_misc, core, os)]

extern crate zaldinar;
extern crate getopts;

use std::env;
use std::old_io::stdio;
use std::ffi::AsOsStr;

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
    let mut args = env::args();
    let program = match args.next() {
        // TODO: some error catching of lossy strings here or not?
        Some(v) => v.as_os_str().to_string_lossy().into_owned(),
        None => match env::current_exe() {
            Ok(path) => path.as_os_str().to_string_lossy().into_owned(),
            Err(e) => {
                print_err!("Warning: failed to find current executable: {}", e);
                "<unknown executable>".to_string()
            },
        }
    };
    let mut opts = getopts::Options::new();
    opts.optopt("c", "config", "set config file name", "FILE");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "version", "print program version");

    // TODO: Bug getopts to use OsString
    let matches = match opts.parse(args.collect::<Vec<String>>()) {
        Ok(v) => v,
        Err(e) => {
            print_err!("{}", e.to_string());
            return;
        }
    };

    if matches.opt_present("help") {
        let brief = format!("Usage: {} [options]", program);
        println!("{}", opts.usage(brief.as_slice()));
        return;
    } else if matches.opt_present("version") {
        println!("zaldinar version {}", zaldinar::VERSION);
        return;
    }

    let current_dir = match env::current_dir() {
        Ok(v) => v,
        Err(e) => {
            print_err!("Warning: failed to get current directory: {}", e);
            Path::new("") // TODO: return here or just not be absolute?
        }
    };

    let config_path = match matches.opt_str("config") {
        Some(v) => {
            let absolute = current_dir.join(&Path::new(v));
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
                env::set_exit_status(1);
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
                env::set_exit_status(1);
            },
        };
    }
}
