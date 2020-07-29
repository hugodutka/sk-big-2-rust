use crate::channels::CHANNEL_MODEL_R;
use crate::events::EventModel;
use crate::log::begin_logging;
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

        thread::spawn(|| begin_logging());

        thread::spawn(move || {
            let mut telnet_server = TelnetServer::new("127.0.0.1", telnet_port);
            telnet_server.start();
        });

        loop {
            match CHANNEL_MODEL_R.recv()? {
                EventModel::TelnetServerCrashed(msg) => {
                    return Err(anyhow!("telnet server crashed\n{}", msg))
                }
                EventModel::UserInput(input) => eprintln!("Got {:?}", input),
            }
        }
    }
}
