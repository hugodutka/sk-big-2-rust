use crate::events::{EventModel, EventTelnet};
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
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::Result;
    use std::sync::{Arc, Mutex};

    lazy_static! {
        static ref TEST_MUTEX: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    }

    /// If your test puts events on global channels it may enter into data races with other tests.
    /// To avoid that, wrap your test in this function like so:
    ///
    /// ```
    /// use crate::channels::tests::channel_test;
    /// use adorn::adorn;
    /// use anyhow::Result;
    ///
    /// #[test]
    /// #[adorn(channel_test)]
    /// fn my_test() -> Result<()> {
    ///     // actual test logic
    /// }
    /// ```
    pub fn channel_test(f: fn() -> Result<()>) -> Result<()> {
        let _guard = TEST_MUTEX.lock().unwrap();
        while let Ok(_) = CHANNEL_LOG_R.try_recv() {}
        while let Ok(_) = CHANNEL_MODEL_R.try_recv() {}
        while let Ok(_) = CHANNEL_TELNET_R.try_recv() {}
        return f();
    }
}
