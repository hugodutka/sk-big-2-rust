use crate::channels::CHANNEL_MODEL_S;
use crate::events::EventModel;
use anyhow::{Context, Result};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

const BUFFER_SIZE: usize = 1024;

pub struct TelnetServer<'a> {
    host: &'a str,
    port: u16,
    write_handle: Option<TcpStream>,
    buffer: [u8; BUFFER_SIZE],
}

impl TelnetServer<'_> {
    pub fn new(host: &str, port: u16) -> TelnetServer {
        TelnetServer {
            host,
            port,
            write_handle: None,
            buffer: [0; BUFFER_SIZE],
        }
    }

    pub fn start(&mut self) {
        match || -> Result<()> {
            let listener = TcpListener::bind((self.host, self.port)).context("bind failed")?;

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        self.write_handle = match stream.try_clone() {
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
    use crate::channels::CHANNEL_MODEL_R;
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

        for _ in 0..10 {
            match TcpStream::connect((SERVER_HOST, SERVER_PORT)) {
                Ok(mut stream) => {
                    stream.write_all(&INPUT)?;
                    break;
                }
                Err(_) => thread::sleep(Duration::from_millis(50)),
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
}
