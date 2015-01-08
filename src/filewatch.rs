use std::os;
use std::thread;

use inotify;

use interface;
use client;
use errors::InitializationError;


pub fn watch_binary(client: interface::IrcInterface)
        -> Result<thread::JoinGuard<()>, InitializationError> {
    let mut watch = try!(inotify::INotify::init());
    let program = match os::self_exe_name() {
        Some(v) => v,
        None => return Err(InitializationError::from_string(
            format!("Failed to get the path of the running executable!"))),
    };

    let filename = match program.filename_str() {
        Some(v) => v.to_string(),
        None => return Err(InitializationError::from_string(
            format!("Failed to get filename from program Path ({})", program.display()))),
    };

    // IN_CLOSE_WRITE, IN_MOVED_TO and IN_CREATE are the only events which modify a file, and also
    // leave a fully intact file that is ready to be executed.
    try!(watch.add_watch(&program.dir_path(),
        inotify::ffi::IN_CLOSE_WRITE |
        inotify::ffi::IN_MOVED_TO |
        inotify::ffi::IN_CREATE
    ));

    let guard = thread::Thread::spawn(move || {
        loop {
            let events = match watch.wait_for_events() {
                Ok(v) => v,
                Err(e) => {
                    warning!("INotify error: {}. Exiting.", e);
                    return;
                },
            };
            for event in events.iter() {
                if event.is_ignored() {
                    warning!(
                        "File watch on binary removed due to a deleted directory or unmounted \
                        filesystem. Exiting watch thread, bot will no longer watch binary for \
                        restarting.");
                }

                if event.is_dir() || filename != event.name {
                    continue;
                }

                client.quit(Some("Updating to latest binary"), client::ExecutingState::Restart);
                return;
            }
        }
    });
    return Ok(guard);
}
