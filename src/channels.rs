use crate::events::Event;
use crossbeam::crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;

lazy_static! {
    static ref CHANNEL_MODEL: (Sender<Event>, Receiver<Event>) = unbounded();
    pub static ref CHANNEL_MODEL_S: Sender<Event> = CHANNEL_MODEL.0.clone();
    pub static ref CHANNEL_MODEL_R: Receiver<Event> = CHANNEL_MODEL.1.clone();
}
