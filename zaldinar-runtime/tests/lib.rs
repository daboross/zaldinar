extern crate chrono;
extern crate log;
extern crate fern;
extern crate zaldinar;
extern crate zaldinar_irclib as irc;
extern crate generated_plugins_crate;

use std::path::Path;
use std::sync::mpsc;
use std::env;
use std::thread;

fn setup_logger() {
    fern::Dispatch::new()
        .level(log::LogLevelFilter::Debug)
        .format(|out, message, record| {
            let now = chrono::Local::now();

            out.finish(format_args!("{}[{}] {}", now.format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(), message));
        })
        .chain(std::io::stdout())
        .apply()
        // let this fail
        .unwrap_or(());
}

fn setup() -> (zaldinar::client::Client, zaldinar::interface::IrcInterface,
        mpsc::Sender<irc::IrcMessage>, mpsc::Receiver<Option<String>>) {
    setup_logger();

    let zaldinar_dir = env::current_dir().unwrap();
    let project_dir = Path::new(&zaldinar_dir).parent()
        .expect("Expected working directory to have parent!");
    let config = zaldinar::config::ClientConfiguration::load_from_file(
        &project_dir.join("default-config.json")).unwrap();

    let plugins = zaldinar::client::PluginRegister::new();

    let (client, interface, dispatch, conn_data_out, conn_data_in) =
        zaldinar::startup::prepare(plugins, config).unwrap();

    thread::spawn(move || {
        dispatch.dispatch_loop();
    });

    return (client, interface, conn_data_out, conn_data_in);
}

#[test]
fn test_setup() {
    let (client, interface, conn_data_out, conn_data_in) = setup();
    // TODO: tests here with input/output
}
