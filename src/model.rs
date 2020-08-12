use crate::channels::CHANNEL_MODEL_R;
use crate::events::EventModel;
use crate::log::begin_logging;
use crate::proxy;
use crate::telnet::TelnetServer;
use crate::ui;
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
        thread::spawn(move || TelnetServer::new("127.0.0.1", telnet_port).start());
        thread::spawn(|| TelnetServer::start_writer());
        thread::spawn(|| proxy::start(("127.0.0.1", 14221)));
        thread::spawn(|| proxy::start_writer());

        loop {
            match CHANNEL_MODEL_R.recv()? {
                EventModel::TelnetServerCrashed(msg) => {
                    return Err(anyhow!("telnet server crashed\n{}", msg))
                }
                EventModel::UserInput(input) => log!("Got {:?}", input),
                EventModel::NewTelnetConnection() => {
                    log!("new connection!");
                    ui::prepare_screen();
                    ui::render("hello!");
                }
                EventModel::ProxyServerCrashed(msg) => {
                    return Err(anyhow!("proxy server crashed\n{}", msg))
                }
                EventModel::ProxyInput(msg) => {
                    log!("new proxy input: {:?}", msg);
                }
            }
        }
    }
}
