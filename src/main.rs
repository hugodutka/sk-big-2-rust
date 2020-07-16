use anyhow::Result;
use std::{panic, process};

mod channels;
mod events;
mod model;
mod telnet;

use model::Model;

fn main() -> Result<()> {
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        orig_hook(info);
        process::exit(1);
    }));

    let model = Model::new(5100);

    model.start()?;

    Ok(())
}
