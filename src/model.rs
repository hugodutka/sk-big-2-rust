use crate::channels::MODEL_R;
use crate::events::Event;
use crate::telnet::TelnetServer;
use anyhow::{anyhow, Result};
use std::thread;

pub struct Model {
    telnet_port: u16,
}

impl Model {
    pub fn new(telnet_port: u16) -> Model {
        Model { telnet_port }
    }

    pub fn start(&self) -> Result<()> {
        let telnet_port = self.telnet_port;

        thread::spawn(move || {
            let mut telnet_server = TelnetServer::new("127.0.0.1".to_string(), telnet_port);
            telnet_server.start();
        });

        loop {
            match MODEL_R.recv()? {
                Event::TelnetServerCrashed(msg) => {
                    return Err(anyhow!("telnet server crashed\n{}", msg))
                }
                Event::UserInput(input) => eprintln!("Got {:?}", input),
            }
        }
    }
}
