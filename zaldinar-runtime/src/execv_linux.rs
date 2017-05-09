use std::os::unix::ffi::OsStrExt;
use std::{env, ffi, ptr, path};

use libc;

use UNKNOWN_EXECUTABLE;

pub fn execv_if_possible(program_path: &path::Path) -> i32 {
    if program_path == path::Path::new(UNKNOWN_EXECUTABLE) {
        print_err!("Couldn't restart using exec: executable unknown!  See previous \"failed to \
            find current executable\" error.");
        return 1;
    }

    // We're just going to exit the program anyways if we succeed or fail, so this function won't do anything other than unwrap IO errors.

    // Get the program as a CString
    let program = ffi::CString::new(program_path.as_os_str().as_bytes()).unwrap();

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
        libc::execv(program.as_ptr(), argv_vec.as_mut_ptr());
    }
    print_err!("Executing using execv failed!");
    return 1;
}
