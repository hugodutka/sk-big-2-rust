use crate::proxy::{IncomingProxyMessage, OutgoingProxyMessage};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
pub enum EventModel {
    NewTelnetConnection(),
    ProxyInput((SocketAddr, IncomingProxyMessage)),
    ProxyServerCrashed(Arc<str>),
    TelnetServerCrashed(Arc<str>),
    UserInput(Arc<[u8]>),
}

#[derive(Debug)]
pub enum EventTelnet {
    Write(Arc<[u8]>),
}

#[derive(Debug)]
pub enum EventProxy {
    Write((SocketAddr, OutgoingProxyMessage)),
}
