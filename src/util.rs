#![macro_use]

macro_rules! continue_on_err {
    ($res:expr, $msg:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => {
                log!("{}: {:?}", $msg, err);
                continue;
            }
        }
    };
}

#[cfg(test)]
pub mod tests {
    use crate::channels::CHANNEL_LOG_R;
    use anyhow::anyhow;
    use rusty_fork::rusty_fork_test;
    use std::time::Duration;

    rusty_fork_test! {
        #[test]
        fn continue_on_err_works() {
            const MSG: &'static str = "something";
            const ERR_MSG: &'static str = "test message";
            let expected_log = format!("{}: {:?}", MSG, anyhow!(ERR_MSG));
            let mut last_i = 2;
            for i in 0..2 {
                match i {
                    0 => (),
                    1 => continue_on_err!(Err(anyhow!(ERR_MSG)), MSG),
                    _ => panic!("this loop shouldn't go that far")
                }
                last_i = i;
            }
            if last_i != 0 {
                panic!("continue did not work");
            }
            let log = CHANNEL_LOG_R.recv_timeout(Duration::from_secs(1)).unwrap();
            if log != expected_log {
                panic!("expected to receive {:?} but got {:?}", expected_log, log);
            }
        }
    }
}
