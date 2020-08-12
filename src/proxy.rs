use crate::channels::{CHANNEL_MODEL_S, CHANNEL_PROXY_R};
use crate::events::{EventModel, EventProxy};
use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use std::convert::TryFrom;
use std::net::{ToSocketAddrs, UdpSocket};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

const HEADER_SIZE: usize = 4;

mod message_codes {
    pub const DISCOVER: u16 = 1;
    pub const IAM: u16 = 2;
    pub const KEEPALIVE: u16 = 3;
    pub const AUDIO: u16 = 4;
    pub const METADATA: u16 = 6;
}

#[derive(Debug)]
pub enum IncomingProxyMessage {
    Audio(Arc<[u8]>),
    IAM(Arc<str>),
    Metadata(Arc<[u8]>),
}

#[derive(Debug)]
pub enum OutgoingProxyMessage {
    Discover(),
    KeepAlive(),
}

lazy_static! {
    static ref SOCKET: Arc<Mutex<Option<UdpSocket>>> = Arc::from(Mutex::new(None));
}

fn parse_msg(msg: &[u8]) -> Result<IncomingProxyMessage> {
    if msg.len() < HEADER_SIZE {
        return Err(anyhow!("message too short"));
    }
    let code: u16;
    let length: u16;
    unsafe {
        let msg_u16 = msg.as_ptr() as *mut u16;
        code = *msg_u16;
        length = *(msg_u16.add(1));
    }
    let content = &msg[HEADER_SIZE..(HEADER_SIZE + length as usize)];
    match code {
        message_codes::IAM => Ok(IncomingProxyMessage::IAM(Arc::from(from_utf8(content)?))),
        message_codes::AUDIO => Ok(IncomingProxyMessage::Audio(Arc::from(content))),
        message_codes::METADATA => Ok(IncomingProxyMessage::Metadata(Arc::from(content))),
        _ => Err(anyhow!("invalid message code: {}", code)),
    }
}

pub fn start<A: ToSocketAddrs>(addr: A) {
    match || -> Result<()> {
        let socket = UdpSocket::bind(&addr)?;
        {
            *SOCKET.lock().unwrap() = Some(socket.try_clone()?);
        }
        let mut buf: [u8; 65535] = [0; 65535];

        loop {
            let (size, src) = match socket.recv_from(&mut buf) {
                Ok(result) => result,
                Err(err) => {
                    log!("failed to receive UDP message: {:?}", err);
                    continue;
                }
            };
            let msg = match parse_msg(&buf[..size]) {
                Ok(result) => result,
                Err(err) => {
                    log!("failed to parse UDP message: {:?}", err);
                    continue;
                }
            };
            CHANNEL_MODEL_S
                .send(EventModel::ProxyInput((src, msg)))
                .unwrap();
        }
    }() {
        Ok(()) => (),
        Err(e) => CHANNEL_MODEL_S
            .send(EventModel::ProxyServerCrashed(Arc::from(e.to_string())))
            .unwrap(),
    }
}

fn prepare_msg(code: u16, content: &[u8]) -> Result<Vec<u8>> {
    let mut msg = vec![0_u8; HEADER_SIZE + content.len()];
    let length = u16::try_from(content.len()).context("content length must fit in u16")?;
    unsafe {
        let msg_u16 = msg.as_mut_ptr() as *mut u16;
        *msg_u16 = code;
        *(msg_u16.add(1)) = length;
    }
    Ok(msg)
}

pub fn start_writer() {
    match || -> Result<()> {
        loop {
            match CHANNEL_PROXY_R.recv().unwrap() {
                EventProxy::Write((addr, msg)) => match SOCKET.lock().unwrap().as_mut() {
                    Some(socket) => {
                        let (code, content) = match msg {
                            OutgoingProxyMessage::Discover() => (message_codes::DISCOVER, &[]),
                            OutgoingProxyMessage::KeepAlive() => (message_codes::KEEPALIVE, &[]),
                        };
                        let buf = match prepare_msg(code, content) {
                            Ok(result) => result,
                            Err(err) => {
                                log!("failed to prepare message: {:?}", err);
                                continue;
                            }
                        };
                        if let Err(err) = socket.send_to(&buf[..], addr) {
                            log!("failed to send message: {:?}", err);
                            continue;
                        }
                    }
                    None => log!("tried to write when socket was None"),
                },
            }
        }
    }() {
        Ok(()) => (),
        Err(e) => CHANNEL_MODEL_S
            .send(EventModel::ProxyServerCrashed(Arc::from(e.to_string())))
            .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_size_is_big_enough() {
        // unsafe code depends on this
        if HEADER_SIZE != 4 {
            panic!("header size must be 4");
        }
    }
}
