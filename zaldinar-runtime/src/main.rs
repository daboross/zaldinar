#![feature(std_misc)] // For ffi::AsOsStr & os::unix::OsStrExt
#![feature(exit_status)] // For env::set_exit_status

macro_rules! print_err {
    ($($arg:tt)*) => (
        {
            use std::io::prelude::*;
            if let Err(e) = write!(&mut ::std::io::stderr(), "{}\n", format_args!($($arg)*)) {
                panic!("Failed to write to stderr.\
                    \nOriginal error output: {}\
                    \nSecondary error writing to stderr: {}", format!($($arg)*), e);
            }
        }
    )
}

extern crate zaldinar;
extern crate getopts;

#[cfg(feature = "binary-filewatch")]
mod execv_linux;

use std::env;
use std::io;
use std::ffi::AsOsStr;
use std::path::PathBuf;
use std::path::Path;

#[cfg(feature = "binary-filewatch")]
use execv_linux::execv_if_possible;

const UNKNOWN_EXECUTABLE: &'static str = "<unknown executable>";

#[cfg(not(feature = "binary-filewatch"))]
fn execv_if_possible(_program_path: &Path) {
    println!("Can't restart using execv on this platform!");
}

fn get_program() -> io::Result<PathBuf> {
    // This joins current_dir() and current_exe(), resulting in an absolute path to of the current
    // executable
    let mut buf = try!(env::current_dir());
    buf.push(&try!(env::current_exe()));
    return Ok(buf);
}


fn main() {
    // Because env::current_exe() will change if the executing file is moved, we need to get the
    // original program path as soon as we start.
    // We have two `program` variables because we still want to use the program gotten from
    // env::args() to print help strings.
    let original_program = match get_program() {
        Ok(v) => v,
        Err(e) => {
            print_err!("Warning: failed to find current executable: {}", e);
            PathBuf::new(UNKNOWN_EXECUTABLE)
        }
    };

    let mut args = env::args();
    let display_program = match args.next() {
        // TODO: some error catching of lossy strings here or not?
        Some(v) => v,
        None => original_program.as_os_str().to_string_lossy().into_owned(),
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
        let brief = format!("Usage: {} [options]", display_program);
        println!("{}", opts.usage(&brief));
        return;
    } else if matches.opt_present("version") {
        println!("zaldinar version {}", zaldinar::VERSION);
        return;
    }

    let current_dir = match env::current_dir() {
        Ok(v) => v,
        Err(e) => {
            print_err!("Warning: failed to get current directory: {}", e);
            PathBuf::new("") // TODO: return here or just not be absolute?
        }
    };

    let config_path = match matches.opt_str("config") {
        Some(v) => {
            let absolute = current_dir.join(&Path::new(&v));
            println!("Using configuration: {}", absolute.display());
            absolute
        },
        None => current_dir.join(Path::new("config.json")),
    };

    loop {
        let config = match zaldinar::ClientConfiguration::load_from_file(&config_path) {
            Ok(v) => v,
            Err(e) => {
                print_err!("Error loading configuration from `{}`: {}",config_path.display(), e);
                env::set_exit_status(1);
                return;
            },
        };

        match zaldinar::run(config) {
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
                execv_if_possible(&original_program);
                return;
            }
            Err(e) => {
                println!("Error running client: {}", e);
                env::set_exit_status(1);
                return;
            },
        };
    }
}
