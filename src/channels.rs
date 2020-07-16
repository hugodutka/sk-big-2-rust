use crate::events::Event;
use crossbeam::crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;

lazy_static! {
    static ref MODEL_CHANNEL: (Sender<Event>, Receiver<Event>) = unbounded();
    pub static ref MODEL_S: Sender<Event> = MODEL_CHANNEL.0.clone();
    pub static ref MODEL_R: Receiver<Event> = MODEL_CHANNEL.1.clone();
}
