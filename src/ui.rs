use crate::channels::CHANNEL_TELNET_S;
use crate::events::EventTelnet;
use std::sync::Arc;

mod telnet_sequence {
    pub const CLEAR_SCREEN: &[u8] = &[27, 91, 72, 27, 91, 50, 74];
    pub const SCREEN_OPTIONS: &[u8] = &[
        255, 253, 34, // do linemode
        255, 250, 34, 1, 0, 255, 240, // linemode options
        255, 251, 1, // will echo
    ];
}

pub fn prepare_screen() {
    CHANNEL_TELNET_S
        .send(EventTelnet::Write(Arc::from(
            telnet_sequence::SCREEN_OPTIONS,
        )))
        .unwrap();
}

pub fn render(text: &str) {
    CHANNEL_TELNET_S
        .send(EventTelnet::Write(Arc::from(
            &[telnet_sequence::CLEAR_SCREEN, text.as_bytes()].concat()[..],
        )))
        .unwrap();
}
