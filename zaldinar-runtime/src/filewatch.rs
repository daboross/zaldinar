use std::{io, env, thread};
use std::time::Duration;
use std::path::Path;

use inotify::{Inotify, watch_mask, event_mask};

use interface;
use client;
use errors::InitializationError;


pub fn watch_binary(client: interface::IrcInterface)
        -> Result<thread::JoinHandle<()>, InitializationError> {
    let mut watch = Inotify::init()?;
    let program = match env::current_exe() {
        Ok(v) => v,
        Err(e) => return Err(InitializationError::from_string(
            format!("Failed to get the path of the running executable: {}", e))),
    };
    let filename = match program.file_name() {
        Some(v) => v.to_os_string(),
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
        watch_mask::CLOSE_WRITE |
        watch_mask::MOVED_TO |
        watch_mask::CREATE
    ));

    let thread = thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        'thread_loop: loop {
            let events = match watch.read_events_blocking(&mut buffer) {
                Ok(v) => v,
                Err(e) => {
                    if e.kind() == io::ErrorKind::Interrupted {
                        continue 'thread_loop;
                    }
                    error!("INotify error: {}. Exiting watch thread, bot will no longer watch \
                        binary for restarting.", e);
                    return;
                },
            };
            for event in events {
                if event.mask.contains(event_mask::IGNORED) {
                    warn!("File watch on binary removed due to a deleted directory or unmounted \
                        filesystem. Exiting watch thread, bot will no longer watch binary for \
                        restarting.");
                }

                if event.mask.contains(event_mask::ISDIR) || Path::new(event.name).file_name() != Some(&filename) {
                    continue;
                }

                info!("Restarting to update to latest binary momentarily.");
                thread::sleep(Duration::from_secs(1));
                client.quit(Some("Updating to latest binary"),
                    client::ExecutingState::RestartExec);
                break 'thread_loop;
            }
        }
        watch.rm_watch(watch_instance).unwrap();
    });
    return Ok(thread);
}
