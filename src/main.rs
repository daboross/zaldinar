#![feature(old_io, io, path, std_misc, os, exit_status)]
#![cfg_attr(target_os = "linux", feature(libc))]
extern crate zaldinar;
extern crate getopts;
#[cfg(target_os = "linux")]
extern crate libc;

use std::env;
use std::io;
use std::old_io::stdio;
use std::ffi::AsOsStr;
#[cfg(target_os = "linux")]
use std::os::unix::OsStrExt;
use std::path::PathBuf;
use std::path::Path;
#[cfg(target_os = "linux")]
use std::ffi;
#[cfg(target_os = "linux")]
use libc::funcs::posix88::unistd;
#[cfg(target_os = "linux")]
use std::ptr;

const UNKNOWN_EXECUTABLE: &'static str = "<unknown executable>";

macro_rules! print_err {
    ($($arg:tt)*) => (
        if let Err(e) = write!(&mut stdio::stderr_raw(), $($arg)*) {
            panic!("Failed to write to stderr.\
                \nOriginal error output: {}\
                \nSecondary error writing to stderr: {}", format!($($arg)*), e);
        }
    )
}

#[cfg(target_os = "linux")]
#[inline]
fn execv_if_possible(program_path: &Path) {

    if program_path == Path::new(UNKNOWN_EXECUTABLE) {
        print_err!("Couldn't restart using exec: executable unknown! See previous \"failed to \
                    find current executable\" error.");
        env::set_exit_status(1);
        return;
    }

    // We're just going to exit the program anyways if we succeed or fail, so this function won't do anything other than unwrap IO errors.

    let program = program_path.as_os_str().to_cstring().unwrap();
    let mut argv_vec = Vec::new();
    for arg in env::args().skip(1) {
        argv_vec.push(ffi::CString::new(arg).unwrap().as_ptr());
    }
    argv_vec.push(ptr::null());

    println!("Executing `{:?}`", program_path);

    unsafe {
        unistd::execv(program.as_ptr(), argv_vec.as_mut_ptr());
    }
    println!("Oh hi, executing didn't work.");
    unsafe {
        unistd::execv(ffi::CString::new("/bin/bash").unwrap().as_ptr(), vec!().as_mut_ptr());
    }
    println!("Oh hi, executing didn't work *again*.");
}

#[cfg(not(target_os = "linux"))]
#[inline]
fn execv_if_possible(_program_path: &Path) {
    println!("Can't execv! Invalid action for this platform.");
}

fn get_program() -> io::Result<PathBuf> {
    // This essentially joins current_dir() and current_exe(), resulting in an absolute path
    // to of the current executable
    let mut buf = try!(env::current_dir());
    buf.push(&try!(env::current_exe()));
    return Ok(buf);
}

fn main() {
    // Because env::current_exe() will change if the executing file is moved, we need to securely
    // get the original program path as soon as we start.
    // We have two `program` variables because we still want to use the program gotten from
    // env::args() to print help strings.
    let original_program = match get_program(){
        Ok(v) => v,
        Err(e) => {
            print_err!("Warning: failed to find current executable: {}", e);
            PathBuf::new(UNKNOWN_EXECUTABLE)
        }
    };

    let mut args = env::args();
    let program = match args.next() {
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
                execv_if_possible(&original_program);
                return;
            }
            Err(e) => {
                println!("Error running client: {}", e);
                env::set_exit_status(1);
            },
        };
    }
}
