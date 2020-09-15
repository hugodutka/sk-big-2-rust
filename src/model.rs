use crate::channels::{CHANNEL_MODEL_R, CHANNEL_MODEL_S};
use crate::events::EventModel;
use crate::log::begin_logging;
use crate::proxy;
use crate::proxy::{IncomingProxyMessage, OutgoingProxyMessage};
use crate::telnet::TelnetServer;
use crate::ui;
use crate::ui::UserInput;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::{max, min};
use std::io::stdout;
use std::io::Write;
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, SystemTime};

lazy_static! {
    static ref METADATA_RE: Regex = Regex::new("StreamTitle='(.*)'").unwrap();
}

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
        thread::spawn(|| loop {
            CHANNEL_MODEL_S.send(EventModel::Tick()).unwrap();
            thread::sleep(Duration::from_secs(1));
        });

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
                EventModel::ProxyInput((addr, msg)) => {
                    let mut proxy = match self.proxies.iter_mut().find(|x| x.addr == addr) {
                        Some(info) => {
                            info.last_contact = SystemTime::now();
                            info
                        }
                        None => {
                            self.proxies.push(ProxyInfo {
                                addr,
                                last_contact: SystemTime::now(),
                                info: "".to_string(),
                                meta: "".to_string(),
                            });
                            self.proxies.last_mut().unwrap()
                        }
                    };
                    match msg {
                        IncomingProxyMessage::Audio(audio) => {
                            if Some(addr) == self.active_proxy {
                                if let Err(err) = stdout().write_all(&*audio) {
                                    log!("could not print audio: {:?}", err);
                                }
                            }
                            PostAction::Idle()
                        }
                        IncomingProxyMessage::Metadata(meta) => {
                            match std::str::from_utf8(&*meta) {
                                Ok("") => (),
                                Ok(text) => {
                                    proxy.meta = match METADATA_RE.captures_iter(text).next() {
                                        Some(cap) => cap[1].to_string(),
                                        None => text.to_string(),
                                    }
                                }
                                Err(err) => log!("could not parse metadata: {:?}", err),
                            }
                            PostAction::Render()
                        }
                        IncomingProxyMessage::IAM(info) => {
                            proxy.info = info.to_string();
                            PostAction::Render()
                        }
                    }
                }
                EventModel::Tick() => {
                    let now = SystemTime::now();
                    let prev_length = self.proxies.len();
                    self.proxies = self
                        .proxies
                        .drain(..)
                        .filter(|x| match now.duration_since(x.last_contact) {
                            Ok(dur) => dur < Duration::from_secs(5),
                            Err(_) => true,
                        })
                        .collect();
                    for p in &self.proxies {
                        proxy::write(&p.addr, OutgoingProxyMessage::KeepAlive());
                    }
                    if prev_length == self.proxies.len() {
                        PostAction::Idle()
                    } else {
                        PostAction::Render()
                    }
                }
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
