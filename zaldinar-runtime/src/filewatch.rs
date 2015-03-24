extern crate libc;
use std::env;
use std::io;
use std::thread;

use inotify;

use interface;
use client;
use errors::InitializationError;

/// This method was taken from
/// https://github.com/rust-lang/rust/blob/3e4be02b80a3dd27bce20870958fe0aef7e7336d
/// /src/libstd/sys/unix/timer.rs#L230
fn sleep(ms: u64) {
    let mut to_sleep = libc::timespec {
        tv_sec: (ms / 1000) as libc::time_t,
        tv_nsec: ((ms % 1000) * 1000000) as libc::c_long,
    };
    while unsafe { libc::nanosleep(&to_sleep, &mut to_sleep) } != 0 {
        if io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            panic!("failed to sleep, but not because of EINTR?");
        }
    }
}

pub fn watch_binary(client: interface::IrcInterface)
        -> Result<thread::JoinHandle, InitializationError> {
    let mut watch = try!(inotify::INotify::init());
    let program = match env::current_exe() {
        Ok(v) => v,
        Err(e) => return Err(InitializationError::from_string(
            format!("Failed to get the path of the running executable: {}", e))),
    };

    let filename = match program.file_name() {
        Some(v) => match v.to_str() {
            Some(v) => v.to_string(),
            None => return Err(InitializationError::from_string(
                format!("Program filename invalid utf8!"))),
        },
        None => return Err(InitializationError::from_string(
            format!("Failed to get filename from program Path ({})", program.display()))),
    };

    let parent_dir = match program.parent() {
        Some(v) => v,
        None => return Err(InitializationError::from_string(
            format!("Couldn't get parent dir of {:?}", program))),
    };

    // IN_CLOSE_WRITE, IN_MOVED_TO and IN_CREATE are the only events which modify a file, and also
    // leave a fully intact file that is ready to be executed.
    let watch_instance = try!(watch.add_watch(&parent_dir,
        inotify::ffi::IN_CLOSE_WRITE |
        inotify::ffi::IN_MOVED_TO |
        inotify::ffi::IN_CREATE
    ));

    let thread = thread::spawn(move || {
        'thread_loop: loop {
            let events = match watch.wait_for_events() {
                Ok(v) => v,
                Err(e) => {
                    error!("INotify error: {}. Exiting.", e);
                    return;
                },
            };
            for event in events {
                if event.is_ignored() {
                    warn!(
                        "File watch on binary removed due to a deleted directory or unmounted \
                        filesystem. Exiting watch thread, bot will no longer watch binary for \
                        restarting.");
                }

                if event.is_dir() || filename != event.name {
                    continue;
                }

                debug!("Event! \"{}\"", event.name);
                if event.is_access() {
                    debug!("\tevent is: access");
                }
                if event.is_modify() {
                    debug!("\tevent is: modify");
                }
                if event.is_attrib() {
                    debug!("\tevent is: attrib");
                }
                if event.is_close_write() {
                    debug!("\tevent is: close_write");
                }
                if event.is_close_nowrite() {
                    debug!("\tevent is: close_nowrite");
                }
                if event.is_open() {
                    debug!("\tevent is: open");
                }
                if event.is_moved_from() {
                    debug!("\tevent is: moved_from");
                }
                if event.is_moved_to() {
                    debug!("\tevent is: moved_to");
                }
                if event.is_create() {
                    debug!("\tevent is: create");
                }
                if event.is_delete() {
                    debug!("\tevent is: delete");
                }
                if event.is_delete_self() {
                    debug!("\tevent is: delete_self");
                }
                if event.is_move_self() {
                    debug!("\tevent is: move_self");
                }
                if event.is_move() {
                    debug!("\tevent is: move");
                }
                if event.is_close() {
                    debug!("\tevent is: close");
                }
                if event.is_dir() {
                    debug!("\tevent is: dir");
                }
                if event.is_unmount() {
                    debug!("\tevent is: unmount");
                }
                if event.is_queue_overflow() {
                    debug!("\tevent is: queue_overflow");
                }
                if event.is_ignored() {
                    debug!("\tevent is: ignored");
                }
                info!("Restarting to update to latest binary momentarily.");
                sleep(1000u64); // 1000ms = 1 second
                client.quit(Some("Updating to latest binary"), client::ExecutingState::Restart);
                break 'thread_loop;
            }
        }
        watch.rm_watch(watch_instance).unwrap();
    });
    return Ok(thread);
}
