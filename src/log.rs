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
