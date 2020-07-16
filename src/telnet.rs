use crate::channels::MODEL_S;
use crate::events::Event;
use anyhow::{Context, Result};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

const BUFFER_SIZE: usize = 1024;

pub struct TelnetServer {
    host: String,
    port: u16,
    write_handle: Option<TcpStream>,
    buffer: [u8; BUFFER_SIZE],
}

impl TelnetServer {
    pub fn new(host: String, port: u16) -> TelnetServer {
        TelnetServer {
            host,
            port,
            write_handle: None,
            buffer: [0; BUFFER_SIZE],
        }
    }

    pub fn start(&mut self) {
        let mut wrapper = || -> Result<()> {
            let listener =
                TcpListener::bind(format!("{}:{}", self.host, self.port)).context("bind failed")?;

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        self.write_handle = match stream.try_clone() {
                            Ok(handle) => Some(handle),
                            Err(err) => {
                                eprintln!("failed to clone a TCP stream: {:?}", err);
                                continue;
                            }
                        };
                        if let Err(err) = self.handle_client(&mut stream) {
                            eprintln!("TCP connection dropped: {:?}", err);
                        }
                    }
                    Err(err) => eprintln!("failed to unpack a new TCP stream: {:?}", err),
                }
            }

            Ok(())
        };

        match wrapper() {
            Ok(()) => (),
            Err(e) => MODEL_S
                .send(Event::TelnetServerCrashed(e.to_string()))
                .unwrap(),
        }
    }

    fn handle_client(&mut self, stream: &mut TcpStream) -> Result<()> {
        loop {
            let read_size = stream.read(&mut self.buffer).context("read failed")?;
            if read_size == 0 {
                return Ok(());
            }
            MODEL_S
                .send(Event::UserInput(self.buffer[0..read_size].to_vec()))
                .unwrap();
        }
    }
}
