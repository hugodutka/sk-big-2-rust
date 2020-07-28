pub enum EventModel {
    UserInput(Vec<u8>),
    TelnetServerCrashed(String),
}
