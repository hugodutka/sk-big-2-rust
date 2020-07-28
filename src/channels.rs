use crate::events::EventModel;
use crossbeam::crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;

lazy_static! {
    static ref CHANNEL_MODEL: (Sender<EventModel>, Receiver<EventModel>) = unbounded();
    pub static ref CHANNEL_MODEL_S: Sender<EventModel> = CHANNEL_MODEL.0.clone();
    pub static ref CHANNEL_MODEL_R: Receiver<EventModel> = CHANNEL_MODEL.1.clone();
    static ref CHANNEL_LOG: (Sender<String>, Receiver<String>) = unbounded();
    pub static ref CHANNEL_LOG_S: Sender<String> = CHANNEL_LOG.0.clone();
    pub static ref CHANNEL_LOG_R: Receiver<String> = CHANNEL_LOG.1.clone();
}
