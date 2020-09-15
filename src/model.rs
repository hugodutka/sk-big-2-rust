use crate::channels::CHANNEL_MODEL_R;
use crate::events::EventModel;
use crate::log::begin_logging;
use crate::proxy;
use crate::proxy::{IncomingProxyMessage, OutgoingProxyMessage};
use crate::telnet::TelnetServer;
use crate::ui;
use crate::ui::UserInput;
use anyhow::{anyhow, Result};
use std::cmp::{max, min};
use std::net::SocketAddr;
use std::thread;
use std::time::SystemTime;

enum PostAction {
    Idle(),
    Render(),
}

pub struct ProxyInfo {
    pub addr: SocketAddr,
    pub info: String,
    pub last_contact: SystemTime,
    pub meta: String,
}

pub struct Model {
    telnet_port: u16,
    input_buf: Vec<u8>,
    cursor_line: i64,
    proxies: Vec<ProxyInfo>,
    active_proxy: Option<SocketAddr>,
}

impl Model {
    pub fn new(telnet_port: u16) -> Model {
        Model {
            telnet_port,
            input_buf: vec![],
            cursor_line: 0,
            proxies: vec![],
            active_proxy: None,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let telnet_port = self.telnet_port;

        thread::spawn(|| begin_logging());
        thread::spawn(move || TelnetServer::new("0.0.0.0", telnet_port).start());
        thread::spawn(|| TelnetServer::start_writer());
        thread::spawn(|| proxy::start("0.0.0.0:0"));
        thread::spawn(|| proxy::start_writer());

        loop {
            let post_action = match CHANNEL_MODEL_R.recv()? {
                EventModel::UserInput(input) => {
                    for byte in input.iter() {
                        match ui::interpret_input(&mut self.input_buf, *byte) {
                            UserInput::Up() => self.cursor_line -= 1,
                            UserInput::Down() => self.cursor_line += 1,
                            UserInput::Select() => match self.cursor_line {
                                0 => proxy::write(
                                    &SocketAddr::from(([255, 255, 255, 255], 16000)),
                                    OutgoingProxyMessage::Discover(),
                                ),
                                _ if self.cursor_line == (self.proxies.len() + 1) as i64 => {
                                    return Ok(());
                                }
                                i => {
                                    let addr = self.proxies[(i - 1) as usize].addr;
                                    self.active_proxy = if self.active_proxy == Some(addr) {
                                        None
                                    } else {
                                        Some(addr)
                                    }
                                }
                            },
                            UserInput::Unrecognized() => (),
                        }
                    }
                    PostAction::Render()
                }
                EventModel::ProxyInput((_, msg)) => match msg {
                    _ => {
                        log!("got msg: {:?}", msg);
                        PostAction::Render()
                    }
                },
                EventModel::NewTelnetConnection() => {
                    ui::prepare_screen();
                    PostAction::Render()
                }
                EventModel::ProxyServerCrashed(msg) => {
                    return Err(anyhow!("proxy server crashed\n{}", msg))
                }
                EventModel::TelnetServerCrashed(msg) => {
                    return Err(anyhow!("telnet server crashed\n{}", msg))
                }
            };
            self.cursor_line = min(max(0, self.cursor_line), (self.proxies.len() + 1) as i64);
            match post_action {
                PostAction::Render() => ui::render(
                    ui::generate_ui(&self.proxies, &self.active_proxy, self.cursor_line).as_str(),
                ),
                PostAction::Idle() => (),
            };
        }
    }
}
