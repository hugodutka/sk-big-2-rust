use crate::channels::CHANNEL_TELNET_S;
use crate::events::EventTelnet;
use crate::model::ProxyInfo;
use std::net::SocketAddr;
use std::sync::Arc;

mod telnet_sequence {
    pub const CLEAR_SCREEN: &[u8] = &[27, 91, 72, 27, 91, 50, 74];
    pub const SCREEN_OPTIONS: &[u8] = &[
        255, 253, 34, // do linemode
        255, 250, 34, 1, 0, 255, 240, // linemode options
        255, 251, 1, // will echo
    ];
}

pub enum UserInput {
    Up(),
    Down(),
    Select(),
    Unrecognized(),
}

pub fn prepare_screen() {
    CHANNEL_TELNET_S
        .send(EventTelnet::Write(Arc::from(
            telnet_sequence::SCREEN_OPTIONS,
        )))
        .unwrap();
}

pub fn generate_ui(
    proxies: &Vec<ProxyInfo>,
    active_proxy: &Option<SocketAddr>,
    cursor_line: i64,
) -> String {
    let mut rows: Vec<String> = vec![];
    rows.push("Szukaj pośrednika".to_string());
    for proxy in proxies {
        rows.push(format!(
            "Pośrednik {}{}",
            proxy.info,
            match active_proxy {
                Some(addr) if *addr == proxy.addr => " *",
                _ => "",
            }
        ))
    }
    rows.push("Koniec".to_string());
    rows.push(
        match proxies.iter().find(|x| Some(x.addr) == *active_proxy) {
            Some(proxy) => proxy.meta.clone(),
            None => "".to_string(),
        },
    );
    rows[cursor_line as usize].push_str(" <-");
    for row in &mut rows {
        row.push_str("\r\n");
    }
    rows.concat()
}

pub fn render(text: &str) {
    CHANNEL_TELNET_S
        .send(EventTelnet::Write(Arc::from(
            &[telnet_sequence::CLEAR_SCREEN, text.as_bytes()].concat()[..],
        )))
        .unwrap();
}

pub fn interpret_input(buf: &mut Vec<u8>, input: u8) -> UserInput {
    let max_sequence_length = 3;
    if max_sequence_length <= buf.len() {
        buf.pop();
    }
    buf.insert(0, input);

    match buf.as_slice() {
        [65, 91, 27, ..] => UserInput::Up(),
        [66, 91, 27, ..] => UserInput::Down(),
        [0, 13, ..] | [10, 13, ..] => UserInput::Select(),
        _ => UserInput::Unrecognized(),
    }
}
