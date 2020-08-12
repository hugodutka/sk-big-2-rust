use crate::channels::{CHANNEL_MODEL_S, CHANNEL_TELNET_R};
use crate::events::{EventModel, EventTelnet};
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

const BUFFER_SIZE: usize = 1024;

lazy_static! {
    static ref WRITE_HANDLE: Arc<Mutex<Option<TcpStream>>> = Arc::from(Mutex::new(None));
}

pub struct TelnetServer<'a> {
    host: &'a str,
    port: u16,
    buffer: [u8; BUFFER_SIZE],
}

impl TelnetServer<'_> {
    pub fn new(host: &str, port: u16) -> TelnetServer {
        TelnetServer {
            host,
            port,
            buffer: [0; BUFFER_SIZE],
        }
    }

    pub fn start(&mut self) {
        match || -> Result<()> {
            let listener = TcpListener::bind((self.host, self.port)).context("bind failed")?;

            for result in listener.incoming() {
                match result {
                    Ok(mut stream) => {
                        *WRITE_HANDLE.lock().unwrap() = Some(continue_on_err!(
                            stream.try_clone(),
                            "failed to clone a TCP stream"
                        ));
                        CHANNEL_MODEL_S
                            .send(EventModel::NewTelnetConnection())
                            .unwrap();
                        if let Err(err) = self.handle_client(&mut stream) {
                            log!("TCP connection dropped: {:?}", err);
                        }
                        *WRITE_HANDLE.lock().unwrap() = None;
                    }
                    Err(err) => log!("failed to unpack a new TCP stream: {:?}", err),
                }
            }

            Ok(())
        }() {
            Ok(()) => (),
            Err(e) => CHANNEL_MODEL_S
                .send(EventModel::TelnetServerCrashed(Arc::from(e.to_string())))
                .unwrap(),
        }
    }

    pub fn start_writer() {
        loop {
            match CHANNEL_TELNET_R.recv().unwrap() {
                EventTelnet::Write(data) => match WRITE_HANDLE.lock().unwrap().as_mut() {
                    Some(handle) => handle
                        .write_all(data.as_ref())
                        .unwrap_or_else(|err| log!("telnet write failure: {:?}", err)),
                    None => log!("tried to write when stream was None"),
                },
            }
        }
    }

    fn handle_client(&mut self, stream: &mut TcpStream) -> Result<()> {
        loop {
            let read_size = stream.read(&mut self.buffer).context("read failed")?;
            if read_size == 0 {
                return Ok(());
            }
            CHANNEL_MODEL_S.send(EventModel::UserInput(Arc::from(&self.buffer[0..read_size])))?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::{CHANNEL_MODEL_R, CHANNEL_TELNET_S};
    use rusty_fork::rusty_fork_test;
    use std::thread;
    use std::time::Duration;

    static SERVER_HOST: &'static str = "localhost";
    static SERVER_PORT: u16 = 16789;

    rusty_fork_test! {
        #[test]
        fn start_sends_crash_event() {
            thread::spawn(|| TelnetServer::new("invalidhost", 0).start());
            match CHANNEL_MODEL_R.recv_timeout(Duration::from_secs(5)) {
                Ok(EventModel::TelnetServerCrashed(_)) => (),
                Ok(event) => panic!(
                    "expected the telnet server to crash, but got {:?}",
                    event
                ),
                Err(e) => panic!("expected the telnet server to crash, {:?}", e),
            }
        }

        #[test]
        fn handle_client_sends_user_input_event() {
            const INPUT: &'static [u8] = &[1, 2, 3, 4, 5];

            thread::spawn(|| TelnetServer::new(SERVER_HOST, SERVER_PORT + 1).start());
            for _ in 0..100 {
                match TcpStream::connect((SERVER_HOST, SERVER_PORT + 1)) {
                    Ok(mut stream) => {
                        stream.write_all(&INPUT).unwrap();
                        break;
                    }
                    Err(_) => thread::sleep(Duration::from_millis(5)),
                }
            }

            match CHANNEL_MODEL_R.recv_timeout(Duration::from_secs(1)) {
                Ok(EventModel::NewTelnetConnection()) => (),
                _ => panic!("expected a new connection event"),
            }
            match CHANNEL_MODEL_R.recv_timeout(Duration::from_secs(1)) {
                Ok(EventModel::UserInput(recv_input)) => match &recv_input[..] {
                    INPUT => (),
                    _ => panic!("wrong user input received: {:?}", recv_input),
                },
                _ => panic!("expected a user input event"),
            }
        }

        #[test]
        fn telnet_writer_reacts_to_events() {
            const INPUT: &'static [u8] = &[6, 7, 8, 9, 10];

            *WRITE_HANDLE.lock().unwrap() = None;

            thread::spawn(|| TelnetServer::new(SERVER_HOST, SERVER_PORT + 2).start());
            thread::spawn(|| TelnetServer::start_writer());

            for _ in 0..100 {
                match TcpStream::connect((SERVER_HOST, SERVER_PORT + 2)) {
                    Ok(mut stream) => {
                        let mut buf: [u8; INPUT.len()] = [0; INPUT.len()];
                        let range = 1000;
                        for i in 0..range {
                            if let Some(_) = WRITE_HANDLE.lock().unwrap().as_ref() {
                                CHANNEL_TELNET_S.send(EventTelnet::Write(Arc::from(INPUT))).unwrap();
                                break;
                            } else {
                                thread::sleep(Duration::from_millis(5));
                            }
                            if i + 1 == range {
                                panic!("could not obtain a write handle");
                            }
                        }
                        stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
                        stream.read_exact(&mut buf).unwrap();
                        assert_eq!(INPUT, &buf[..]);
                        return ();
                    }
                    Err(_) => thread::sleep(Duration::from_millis(5)),
                }
            }

            panic!("failed to connect to tcp stream");
        }
    }
}
