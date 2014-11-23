#![feature(globs)]
#![feature(phase)]
#[phase(plugin)]

extern crate regex_macros;
extern crate regex;

use std::io::*;
use std::sync::atomic::*;
use std::sync::{Arc, RWLock};
use std::ascii::AsciiExt;
use regex::Regex;
use std::collections::HashMap;

pub struct IrcClient {
    socket: TcpStream,
    reader: Option<BufferedReader<TcpStream>>,
    reader_created: Arc<AtomicBool>,
    pub permitted: Regex,
    listeners: Arc<RWLock<HashMap<String, Vec<|&mut IrcEvent|:'static>>>>
}

impl IrcClient {
    pub fn new(addr: &str) -> IrcClient {
        return IrcClient {
            socket: TcpStream::connect(addr).unwrap(),
            reader: None,
            reader_created: Arc::new(AtomicBool::new(false)),
            permitted: regex!(r"^Dabo[^!]*![^@]*@me.dabo.guru$"),
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

    pub fn add_listener(&mut self, command: &str, f: |&mut IrcEvent|:'static) {
        let command_string = command.into_string().to_ascii_lower();
        let mut listener_map = self.listeners.write();
        {
            // I can't use a match here because then I would be borrowing listener_map mutably twice?
            // Once for the match statement, and a second time inside the None branch
            if listener_map.contains_key(&command_string) {
                listener_map.get_mut(&command_string).expect("Wat").push(f);
            } else {
                listener_map.insert(command_string, vec!(f));
            }
        }
        listener_map.downgrade();
    }

    fn init_reader(&mut self) {
        if self.reader_created.swap(true, Ordering::Relaxed) {
            panic!("Reader already created");
        }
        self.reader = Some(BufferedReader::new(self.socket.clone()));
    }

    fn read_line(&mut self) -> Result<String, IoError> {
        let reader = self.reader.as_mut().unwrap();
        let whole_line = try!(reader.read_line());
        return Ok(whole_line.trim_right().into_string())
    }

    fn spawn_reading_thread(mut self) {
        spawn(proc() {
            self.init_reader();
            loop {
                let input = match self.read_line() {
                    Ok(v) => v,
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
                };
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
                    let mut command_listeners = match listener_map.get_mut(&command.to_ascii_lower()) {
                        Some(v) => v,
                        None => continue
                    };

                    for listener in command_listeners.iter_mut() {
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
            reader: None,
            reader_created: self.reader_created.clone(),
            permitted: self.permitted.clone(),
            listeners: self.listeners.clone()
        }
    }

    fn clone_from(&mut self, source: &IrcClient) {
        self.socket = source.socket.clone();
        self.reader = None;
        self.reader_created = source.reader_created.clone();
        self.permitted = source.permitted.clone();
        self.listeners = source.listeners.clone();
    }
}

// TODO: Make all of these variables private and use methods to access them
pub struct IrcEvent<'a> {
    pub client: &'a mut IrcClient,
    pub command: &'a str,
    pub args: &'a [&'a str],
    pub mask: Option<&'a str>
}

impl <'a> IrcEvent<'a> {
    pub fn new<'b>(client: &'b mut IrcClient, command: &'b str, args: &'b [&'b str], mask: Option<&'b str>) -> IrcEvent<'b> {
        return IrcEvent {
            client: client,
            command: command,
            args: args,
            mask: mask
        }
    }
}
