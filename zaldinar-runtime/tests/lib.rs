extern crate zaldinar_irclib as irc;
extern crate zaldinar;
extern crate time;
extern crate fern;
#[macro_use]
extern crate log;
extern crate generated_plugins_crate;

use std::path::Path;
use std::sync::mpsc;
use std::env;
use std::thread;

fn setup_logger() {
    let log_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            return format!("[{}][{:?}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(),
                level, msg);
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Debug,
    };
    match fern::init_global_logger(log_config, log::LogLevelFilter::Debug) {
        Err(fern::InitError::SetLoggerError(_)) | Ok(()) => (),
        Err(e) => panic!("Setting up logger failed: {}", e),
    };
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
