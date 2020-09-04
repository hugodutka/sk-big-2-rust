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

#[derive(Debug, Eq, PartialEq)]
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
            let (size, src) =
                continue_on_err!(socket.recv_from(&mut buf), "failed to receive UDP message");

            let msg = continue_on_err!(parse_msg(&buf[..size]), "failed to parse UDP message");

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
    for (i, val) in content.iter().enumerate() {
        msg[HEADER_SIZE + i] = *val;
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
                        let buf = continue_on_err!(
                            prepare_msg(code, content),
                            "failed to prepare message"
                        );
                        continue_on_err!(socket.send_to(&buf[..], addr), "failed to send message");
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
    use crate::channels::*;
    use crate::events::EventModel;
    use crate::log::begin_logging;
    use rusty_fork::rusty_fork_test;
    use std::net::ToSocketAddrs;
    use std::thread;
    use std::time::Duration;

    static SERVER_HOST: &'static str = "localhost";
    static SERVER_PORT: u16 = 15789;

    #[test]
    fn header_size_is_big_enough() {
        // unsafe code depends on this
        if HEADER_SIZE != 4 {
            panic!("header size must be 4");
        }
    }

    #[test]
    fn prepare_msg_constructs_valid_messages() {
        let code: u16 = 67;
        let content = [2; 82];
        let msg = prepare_msg(code, &content).unwrap();
        unsafe {
            let msg_u16 = msg.as_ptr() as *const u16;
            assert!(*msg_u16 == code);
            assert!(*(msg_u16.add(1)) == content.len() as u16);
            let msg_u8 = msg.as_ptr() as *const u8;
            for (offset, val) in content.iter().enumerate() {
                assert!(*(msg_u8.add(HEADER_SIZE + offset)) == *val);
            }
        }
    }

    #[test]
    fn parse_msg_accepts_valid_messages() {
        let cases = [
            (
                message_codes::IAM,
                "hello".as_bytes(),
                IncomingProxyMessage::IAM(Arc::from("hello")),
            ),
            (
                message_codes::AUDIO,
                &[2; 4],
                IncomingProxyMessage::Audio(Arc::from([2; 4])),
            ),
            (
                message_codes::METADATA,
                &[2; 4],
                IncomingProxyMessage::Metadata(Arc::from([2; 4])),
            ),
        ];
        for (code, content, expected_msg) in cases.iter() {
            let msg = parse_msg(&prepare_msg(*code, content).unwrap()[..]).unwrap();
            assert_eq!(msg, *expected_msg);
        }
    }

    #[test]
    fn parse_msg_does_not_accept_invalid_msg() {
        let msg = parse_msg(&prepare_msg(32, &[]).unwrap()[..]);
        match msg {
            Err(_) => (),
            _ => panic!("message should not be parsed"),
        }
    }

    rusty_fork_test! {
        #[test]
        fn server_processes_message() {
            thread::spawn(|| start((SERVER_HOST, SERVER_PORT)));
            let socket = UdpSocket::bind((SERVER_HOST, SERVER_PORT + 1)).unwrap();
            let msg_content = [];
            let msg = prepare_msg(message_codes::AUDIO, &msg_content).unwrap();

            let mut loops = 0;
            while let Err(_) = socket.send_to(&msg[..], (SERVER_HOST, SERVER_PORT)) {
                thread::sleep(Duration::from_millis(5));
                loops += 1;
                if loops > 500 {
                    panic!("cant send the message!");
                }
            }

            match CHANNEL_MODEL_R.recv_timeout(Duration::from_secs(5)) {
                Ok(EventModel::ProxyInput((_, IncomingProxyMessage::Audio(content)))) =>
                    assert_eq!(*content, msg_content),
                result => panic!("expected to receive an audio message but got {:?}", result),
            }
        }

        #[test]
        fn writer_sends_message() {
            thread::spawn(begin_logging);
            thread::spawn(|| start((SERVER_HOST, SERVER_PORT + 2)));
            thread::spawn(|| start_writer());

            let mut loops = 0;
            loop {
                loops += 1;
                if loops > 1000 {
                    panic!("timeout");
                }
                let socket_present;
                {
                    socket_present = SOCKET.lock().unwrap().is_some();
                }
                if socket_present {
                    break;
                } else {
                    thread::sleep(Duration::from_millis(5));
                }
            }

            let socket_addr = (SERVER_HOST, SERVER_PORT + 3)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap();
            let socket = UdpSocket::bind(socket_addr).unwrap();

            let msg = prepare_msg(message_codes::DISCOVER, &[]).unwrap();
            CHANNEL_PROXY_S.send(
                EventProxy::Write((socket_addr, OutgoingProxyMessage::Discover()))
            ).unwrap();

            let mut buf = [0; 65535];
            socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            match socket.recv(&mut buf) {
                Ok(msg_size) => {
                    assert_eq!(msg_size, msg.len());
                    assert_eq!(buf[..msg_size], msg[..]);
                },
                result => panic!("expected to receive a message but got {:?}", result),
            }
        }
    }
}
