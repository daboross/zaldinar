#![feature(globs)]

extern crate serialize;

use std::io::*;
use std::sync::atomic::*;
use std::sync::{Arc, RWLock};
use std::ascii::AsciiExt;
use std::collections::HashMap;

pub struct IrcClient {
    socket: TcpStream,
    reader_started: Arc<AtomicBool>,
    listeners: Arc<RWLock<HashMap<String, Vec<|&mut IrcEvent|:'static>>>>
}

impl IrcClient {
    pub fn new(addr: &str) -> IrcClient {
        return IrcClient {
            socket: TcpStream::connect(addr).unwrap(),
            reader_started: Arc::new(AtomicBool::new(false)),
            listeners: Arc::new(RWLock::new(HashMap::new()))
        }
    }

    pub fn send(&mut self, command: &str) {
        println!("Sending: {}", command)
        self.socket.write(command.as_bytes()).ok().expect("Failed to write to stream");
        self.socket.write(b"\n").ok().expect("Failed to write to stream");
        self.socket.flush().ok().expect("Failed to flush stream");
    }

    pub fn start_receiving(&self) {
        let reader = self.clone();
        reader.spawn_reading_thread();
    }

    pub fn add_listener(&mut self, irc_command: &str, f: |&mut IrcEvent|:'static) {
        let command_string = irc_command.into_string().to_ascii_lower();
        let mut listener_map = self.listeners.write();
        {
            // I can't use a match here because then I would be borrowing listener_map mutably twice?
            // Once for the match statement, and a second time inside the None branch
            if listener_map.contains_key(&command_string) {
                listener_map.get_mut(&command_string).expect("Honestly, this won't happen.").push(f);
            } else {
                listener_map.insert(command_string, vec!(f));
            }
        }
        listener_map.downgrade();
    }

    fn spawn_reading_thread(self) {
        if self.reader_started.swap(true, Ordering::Relaxed) {
            panic!("Reader already started");
        }
        spawn(proc() {
            let mut reader = BufferedReader::new(self.socket.clone());
            loop {
                let whole_input = match reader.read_line() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
                };
                let input = whole_input.trim_right();
                let message_split: Vec<&str> = input.split(' ').collect();
                let (command, args, possible_mask) = if message_split[0].starts_with(":") {
                    (message_split[1], message_split.slice_from(2), Some(message_split[0].slice_from(1)))
                } else {
                    (message_split[0], message_split.slice_from(1), None)
                };
                match possible_mask {
                    Some(v) => println!("Received {} from {}: {}", command, v, args.connect(" ")),
                    None => println!("Received {}: {}", command, args.connect(" "))
                }
                let shared_self = &mut self.clone();
                let event = &mut IrcEvent::new(shared_self, command, args, possible_mask);
                let mut listener_map = self.listeners.write();
                {
                    let mut listeners = match listener_map.get_mut(&EventType::IrcRaw(command.to_ascii_lower())) {
                        Some(v) => v,
                        None => continue
                    };

                    for listener in listeners.iter_mut() {
                        (*listener)(event);
                    }
                }
                listener_map.downgrade();
            }
        });
    }
}

impl Clone for IrcClient {
    fn clone(&self) -> IrcClient {
        return IrcClient {
            socket: self.socket.clone(),
            reader_started: self.reader_started.clone(),
            listeners: self.listeners.clone()
        }
    }

    fn clone_from(&mut self, source: &IrcClient) {
        self.socket = source.socket.clone();
        self.reader_started = source.reader_started.clone();
        self.listeners = source.listeners.clone();
    }
}


// TODO: Make all of these variables private and use methods to access them
pub struct RawIrcEvent<'a> {
    pub client: &'a mut IrcClient,
    pub command: &'a str,
    pub args: &'a [&'a str],
    pub mask: Option<&'a str>
}

impl <'a> RawIrcEvent<'a> {
    pub fn new(client: &'a mut IrcClient, command: &'a str, args: &'a [&'a str], mask: Option<&'a str>) -> IrcEvent<'a> {
        return IrcEvent {
            client: client,
            command: command,
            args: args,
            mask: mask
        }
    }
}
