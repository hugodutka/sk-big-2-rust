use crate::channels::CHANNEL_MODEL_S;
use crate::events::EventModel;
use anyhow::{anyhow, Result};
use std::net::{ToSocketAddrs, UdpSocket};
use std::sync::Arc;

const HEADER_SIZE: usize = 4;

#[derive(Debug)]
pub enum ProxyMessage {
    Audio(Arc<[u8]>),
    IAM(),
    KeepAlive(),
    Metadata(),
}

fn parse_msg(msg: &[u8]) -> Result<ProxyMessage> {
    if msg.len() < HEADER_SIZE {
        return Err(anyhow!("message too short"));
    }
    let code = msg[0];
    let length = ((msg[0] as u16) << 8) | msg[1] as u16;
    let content = &msg[HEADER_SIZE..(HEADER_SIZE + length as usize)];
    match code {
        2 => Ok(ProxyMessage::IAM()),
        3 => Ok(ProxyMessage::KeepAlive()),
        4 => Ok(ProxyMessage::Audio(Arc::from(content))),
        6 => Ok(ProxyMessage::Metadata()),
        _ => Err(anyhow!("invalid message code")),
    }
}

pub fn start<A: ToSocketAddrs>(addr: A) {
    match || -> Result<()> {
        let socket = UdpSocket::bind(&addr)?;
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
