#![feature(globs)]

use std::io::*;
use std::sync::atomic::*;
use std::sync::Arc;

pub struct IrcClient {
    socket: TcpStream,
    reader: Option<BufferedReader<TcpStream>>,
    reader_created: Arc<AtomicBool>
}

impl IrcClient {
    pub fn new(addr: &str) -> IrcClient {
        return IrcClient {
            socket: TcpStream::connect(addr).unwrap(),
            reader: None,
            reader_created: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn send(&mut self, command: &str) {
        println!("Sending: {}", command)
        self.socket.write(command.as_bytes()).ok().expect("Failed to write to stream");
        self.socket.write(b"\n").ok().expect("Failed to write to stream");
    }

    pub fn init_reader(&mut self) {
        if self.reader_created.swap(true, Ordering::Relaxed) {
            panic!("Reader already created");
        }
        self.reader = Some(BufferedReader::new(self.socket.clone()));
    }

    pub fn read_line(&mut self) -> Result<String, IoError> {
        let reader = self.reader.as_mut().unwrap();
        return reader.read_line()
    }
}

impl Clone for IrcClient {
    fn clone(&self) -> IrcClient {
        return IrcClient {
            socket: self.socket.clone(),
            reader: None,
            reader_created: self.reader_created.clone()
        }
    }

    fn clone_from(&mut self, source: &IrcClient) {
        self.socket = source.socket.clone();
        self.reader = None;
        self.reader_created = self.reader_created.clone();
    }
}
