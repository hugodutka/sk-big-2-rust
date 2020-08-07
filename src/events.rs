use crate::proxy::ProxyMessage;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
pub enum EventModel {
    NewTelnetConnection(),
    ProxyInput((SocketAddr, ProxyMessage)),
    ProxyServerCrashed(Arc<str>),
    TelnetServerCrashed(Arc<str>),
    UserInput(Arc<[u8]>),
}

#[derive(Debug)]
pub enum EventTelnet {
    Write(Arc<[u8]>),
}
