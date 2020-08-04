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
                        *WRITE_HANDLE.lock().unwrap() = match stream.try_clone() {
                            Ok(handle) => Some(handle),
                            Err(err) => {
                                log!("failed to clone a TCP stream: {:?}", err);
                                continue;
                            }
                        };
                        if let Err(err) = self.handle_client(&mut stream) {
                            log!("TCP connection dropped: {:?}", err);
                        }
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
    use crate::channels::tests::channel_test;
    use crate::channels::{CHANNEL_MODEL_R, CHANNEL_TELNET_S};
    use adorn::adorn;
    use anyhow::{anyhow, Result};
    use std::thread;
    use std::time::Duration;

    static SERVER_HOST: &'static str = "localhost";
    static SERVER_PORT: u16 = 16789;

    #[test]
    #[adorn(channel_test)]
    fn start_sends_crash_event() -> Result<()> {
        thread::spawn(|| TelnetServer::new("invalidhost", 0).start());
        match CHANNEL_MODEL_R.recv_timeout(Duration::from_secs(5)) {
            Ok(EventModel::TelnetServerCrashed(_)) => Ok(()),
            Ok(event) => Err(anyhow!(
                "expected the telnet server to crash, but got {:?}",
                event
            )),
            Err(e) => Err(anyhow!("expected the telnet server to crash, {:?}", e)),
        }
    }

    #[test]
    #[adorn(channel_test)]
    fn handle_client_sends_user_input_event() -> Result<()> {
        const INPUT: &'static [u8] = &[1, 2, 3, 4, 5];

        thread::spawn(|| TelnetServer::new(SERVER_HOST, SERVER_PORT).start());

        for _ in 0..100 {
            match TcpStream::connect((SERVER_HOST, SERVER_PORT)) {
                Ok(mut stream) => {
                    stream.write_all(&INPUT)?;
                    break;
                }
                Err(_) => thread::sleep(Duration::from_millis(5)),
            }
        }

        match CHANNEL_MODEL_R.recv_timeout(Duration::from_secs(5)) {
            Ok(EventModel::UserInput(recv_input)) => match &recv_input[..] {
                INPUT => Ok(()),
                _ => Err(anyhow!(format!(
                    "wrong user input received: {:?}",
                    recv_input
                ))),
            },
            _ => Err(anyhow!("expected a user input event")),
        }
    }

    #[test]
    #[adorn(channel_test)]
    fn telnet_writer_reacts_to_events() -> Result<()> {
        const INPUT: &'static [u8] = &[6, 7, 8, 9, 10];

        *WRITE_HANDLE.lock().unwrap() = None;

        thread::spawn(|| TelnetServer::new(SERVER_HOST, SERVER_PORT).start());
        thread::spawn(|| TelnetServer::start_writer());

        for _ in 0..100 {
            match TcpStream::connect((SERVER_HOST, SERVER_PORT)) {
                Ok(mut stream) => {
                    let mut buf: [u8; INPUT.len()] = [0; INPUT.len()];
                    for i in 0..100 {
                        if let Some(_) = WRITE_HANDLE.lock().unwrap().as_ref() {
                            CHANNEL_TELNET_S.send(EventTelnet::Write(Arc::from(INPUT)))?;
                            break;
                        }
                        if i == 99 {
                            return Err(anyhow!("could not obtain a write handle"));
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
                    stream.read_exact(&mut buf)?;
                    assert_eq!(INPUT, &buf[..]);
                    return Ok(());
                }
                Err(_) => thread::sleep(Duration::from_millis(5)),
            }
        }

        return Err(anyhow!("failed to connect to tcp stream"));
    }
}
