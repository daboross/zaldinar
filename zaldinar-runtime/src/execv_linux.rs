extern crate libc;

use std::env;
use std::ffi;
use std::ffi::AsOsStr;
use std::ptr;
use std::path;
use std::os::unix::prelude::OsStrExt;

use self::libc::funcs::posix88::unistd;

use UNKNOWN_EXECUTABLE;

pub fn execv_if_possible(program_path: &path::Path) {
    if program_path == path::Path::new(UNKNOWN_EXECUTABLE) {
        print_err!("Couldn't restart using exec: executable unknown!  See previous \"failed to \
            find current executable\" error.");
        env::set_exit_status(1);
        return;
    }

    // We're just going to exit the program anyways if we succeed or fail, so this function won't do anything other than unwrap IO errors.

    // Get the program as a CString
    let program = program_path.as_os_str().to_cstring().unwrap();

    // Argument vector, passed to execv.
    let mut argv_vec = Vec::new();
    // Printable argument vector, used for printing arguments before executing.
    let mut printable_args = Vec::new();

    // We don't use skip(1) on env::args() here, because the execv() needs the first argument to
    // be the program, just like env::args().
    for arg in env::args() {
        // Just use &*arg so that printable_args can then have the ownership.
        argv_vec.push(ffi::CString::new(&*arg).unwrap().as_ptr());
        printable_args.push(arg);
    }
    // Push a null pointer so that argv_vec is null terminated for execv.
    argv_vec.push(ptr::null());

    println!("Executing `{:?}` (arguments: `{:?}`", program_path, printable_args);

    unsafe {
        unistd ::execv(program.as_ptr(), argv_vec.as_mut_ptr());
    }
    println!("Executing using execv failed!");
    env::set_exit_status(1);
}
