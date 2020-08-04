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
    use rusty_fork::rusty_fork_test;
    use std::time::Duration;

    rusty_fork_test! {
        #[test]
        fn log_sends_event() {
            const MSG: &'static str = "test message";
            log!("{}", MSG);
            match CHANNEL_LOG_R.recv_timeout(Duration::from_secs(1)).unwrap().as_str() {
                MSG => (),
                other => panic!("expected to receive {:?} but got {:?}", MSG, other),
            }
        }
    }
}
