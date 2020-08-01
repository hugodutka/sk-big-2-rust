#![macro_use]
use crate::channels::CHANNEL_LOG_R;

macro_rules! log {
    ($($arg:tt)*) => {
        crate::channels::CHANNEL_LOG_S.send(format!($($arg)*)).unwrap()
    };
}

pub fn begin_logging() {
    loop {
        eprintln!("{}", CHANNEL_LOG_R.recv().unwrap());
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::channels::tests::channel_test;
    use adorn::adorn;
    use anyhow::{anyhow, Result};
    use std::time::Duration;

    #[test]
    #[adorn(channel_test)]
    fn log_sends_event() -> Result<()> {
        const MSG: &'static str = "test message";
        log!("{}", MSG);
        match CHANNEL_LOG_R.recv_timeout(Duration::from_secs(1))?.as_str() {
            MSG => Ok(()),
            other => Err(anyhow!("expected to receive {:?} but got {:?}", MSG, other)),
        }
    }
}
