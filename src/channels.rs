use crate::events::{EventModel, EventProxy, EventTelnet};
use crossbeam::crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;

lazy_static! {
    static ref CHANNEL_MODEL: (Sender<EventModel>, Receiver<EventModel>) = unbounded();
    pub static ref CHANNEL_MODEL_S: Sender<EventModel> = CHANNEL_MODEL.0.clone();
    pub static ref CHANNEL_MODEL_R: Receiver<EventModel> = CHANNEL_MODEL.1.clone();
    static ref CHANNEL_LOG: (Sender<String>, Receiver<String>) = unbounded();
    pub static ref CHANNEL_LOG_S: Sender<String> = CHANNEL_LOG.0.clone();
    pub static ref CHANNEL_LOG_R: Receiver<String> = CHANNEL_LOG.1.clone();
    static ref CHANNEL_TELNET: (Sender<EventTelnet>, Receiver<EventTelnet>) = unbounded();
    pub static ref CHANNEL_TELNET_S: Sender<EventTelnet> = CHANNEL_TELNET.0.clone();
    pub static ref CHANNEL_TELNET_R: Receiver<EventTelnet> = CHANNEL_TELNET.1.clone();
    static ref CHANNEL_PROXY: (Sender<EventProxy>, Receiver<EventProxy>) = unbounded();
    pub static ref CHANNEL_PROXY_S: Sender<EventProxy> = CHANNEL_PROXY.0.clone();
    pub static ref CHANNEL_PROXY_R: Receiver<EventProxy> = CHANNEL_PROXY.1.clone();
}
