pub enum Event {
    UserInput(Vec<u8>),
    TelnetServerCrashed(String),
}
