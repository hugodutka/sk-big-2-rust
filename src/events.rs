use std::sync::Arc;

#[derive(Debug)]
pub enum EventModel {
    UserInput(Arc<[u8]>),
    TelnetServerCrashed(Arc<str>),
    NewTelnetConnection(),
}

#[derive(Debug)]
pub enum EventTelnet {
    Write(Arc<[u8]>),
}
