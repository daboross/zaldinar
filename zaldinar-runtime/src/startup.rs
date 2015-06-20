use std::sync::mpsc;

use generated_plugins_crate;
use time;
use fern;
use log;

use errors::ThrowInitError;
use plugins;
use interface;
use config;
use dispatch;
use irc;
use client;
#[cfg(feature = "binary-filewatch")]
use filewatch;

#[cfg(feature = "binary-filewatch")]
pub fn start_file_watch(client: &client::Client, interface: &interface::IrcInterface) {
    if client.watch_binary {
        if let Err(e) = filewatch::watch_binary(interface.clone()) {
            warn!("Failed to start binary watch thread: {}", e);
        }
    }
}

#[cfg(not(feature = "binary-filewatch"))]
pub fn start_file_watch(_client: &client::Client, _interface: &interface::IrcInterface) {
    // TODO: Maybe support this?
}

fn setup_logger(config: &config::ClientConfiguration) -> Result<(), ThrowInitError> {
    let level = match config.log_level.parse() {
        Ok(v) => v,
        Err(_) => throw_new!(format!("Failed to parse log_level '{}'", config.log_level)),
    };
    let config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            return format!("[{}][{:?}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(),
                level, msg);
        }),
        output: vec![fern::OutputConfig::stdout(), fern::OutputConfig::file(&config.log_file)],
        level: level,
    };
    return match fern::init_global_logger(config, log::LogLevelFilter::Info) {
        Err(fern::InitError::SetLoggerError(_)) => Ok(()),
        Ok(()) => Ok(()),
        Err(e) => throw_new!(e),
    };
}

/// Prepares for startup, performing all operations except for sending the initial on-connect
/// commands, connecting to the IRC server, starting the file watcher and starting the dispatch
/// loop.
///
/// Returns `(client, interface, dispatch, irc_data_in, irc_data_out)`
pub fn prepare(mut plugins: client::PluginRegister, config: config::ClientConfiguration)
        -> Result<(client::Client, interface::IrcInterface, dispatch::Dispatch,
            mpsc::Sender<irc::IrcMessage>, mpsc::Receiver<Option<String>>), ThrowInitError> {
    // Register built-in plugins
    plugins::register_plugins(&mut plugins);
    generated_plugins_crate::register(&mut plugins);

    let client = client::Client::new(plugins, config);

    let (data_out, connection_data_in) = mpsc::channel();
    let (connection_data_out, data_in) = mpsc::channel();

    let interface = up!(interface::IrcInterface::new(data_out, client.clone()));

    // Create dispatch, and start the worker threads for plugin execution
    let dispatch = dispatch::Dispatch::new(interface.clone(), client.clone(), data_in);

    return Ok((client, interface, dispatch, connection_data_out, connection_data_in));
}

pub fn run(config: config::ClientConfiguration)
        -> Result<client::ExecutingState, ThrowInitError> {
    let result = up!(run_with_plugins(config, client::PluginRegister::new()));

    Ok(result)
}

pub fn run_with_plugins(config: config::ClientConfiguration, plugins: client::PluginRegister)
        -> Result<client::ExecutingState, ThrowInitError> {

    up!(setup_logger(&config));

    let (client, interface, dispatch, conn_data_out, conn_data_in) = up!(prepare(plugins, config));

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

    up!(irc::connect(&client.address, conn_data_out, conn_data_in, client.clone()));

    // This statement will run until the bot exists
    dispatch.dispatch_loop();

    let done = {
        let state = throw!(client.state().read());
        state.done_executing
    };

    return Ok(done);
}
