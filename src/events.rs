use std::sync::Arc;

#[derive(Debug)]
pub enum EventModel {
    UserInput(Arc<[u8]>),
    TelnetServerCrashed(Arc<str>),
}
