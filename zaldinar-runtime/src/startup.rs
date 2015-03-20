use std::sync;
use std::sync::mpsc;
use std::path;

use generated_plugins_crate;
use chrono;
use fern;
use log;

use errors::InitializationError;
use plugins;
use interface;
use config;
use dispatch;
use irc;
use client;
#[cfg(target_os = "linux")]
use filewatch;

#[cfg(target_os = "linux")]
fn start_file_watch(client: &sync::Arc<client::Client>, interface: &interface::IrcInterface) {
    if client.watch_binary {
        if let Err(e) = filewatch::watch_binary(interface.clone()) {
            warn!("Failed to start binary watch thread: {}", e);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn start_file_watch(_client: &sync::Arc<client::Client>, _interface: &interface::IrcInterface) {
    // TODO: Maybe support this?
}

fn setup_logger(config: &config::ClientConfiguration) -> Result<(), fern::InitError> {
    let config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            return format!("[{}][{:?}] {}", chrono::Local::now().format("%Y-%m-%d][%H:%M:%S"),
                level, msg);
        }),
        output: vec![fern::OutputConfig::Stdout, fern::OutputConfig::File(
                                                    path::PathBuf::new(&config.log_file))],
        level: log::LogLevelFilter::Info,
    };
    return match fern::init_global_logger(config, log::LogLevelFilter::Info) {
        Err(fern::InitError::SetLoggerError(_)) => Ok(()),
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    };
}

pub fn run(config: config::ClientConfiguration)
        -> Result<client::ExecutingState, InitializationError> {
    run_with_plugins(config, client::PluginRegister::new())
}

pub fn run_with_plugins(config: config::ClientConfiguration, mut plugins: client::PluginRegister)
        -> Result<client::ExecutingState, InitializationError> {

    try!(setup_logger(&config));

    // Register built-in plugins
    plugins::register_plugins(&mut plugins);
    generated_plugins_crate::register(&mut plugins);

    let client = sync::Arc::new(client::Client::new(plugins, config));

    let (data_out, connection_data_in) = mpsc::channel();
    let (connection_data_out, data_in) = mpsc::channel();

    let interface = try!(interface::IrcInterface::new(data_out, client.clone()));

    // Load file watcher
    start_file_watch(&client, &interface);

    // Send PASS, NICK and USER, the initial IRC commands. Because an IrcConnection hasn't been
    // created to receive these yet, they will just go on hold and get sent as soon as the
    // IrcConnection connects.
    if let Some(ref pass) = client.password {
        interface.send_command::<&str, &str>("PASS", &[&pass]);
    }
    interface.send_command::<&str, &str>("NICK", &[&client.nick]);
    interface.send_command::<&str, &str>("USER", &[&client.user, "0", "*",
        &format!(":{}", client.real_name)]);

    try!(irc::connect(&client.address, connection_data_out, connection_data_in, client.clone()));

    // Create dispatch, and start the worker threads for plugin execution
    let dispatch = dispatch::Dispatch::new(interface, client.clone(), data_in);

    // This statement will run until the bot exists
    if let Err(..) = dispatch.start_dispatch_loop() {
        error!("Dispatch loop panicked!");
    }

    let done = {
        let state = try!(client.state.read());
        state.done_executing
    };

    return Ok(done);
}
