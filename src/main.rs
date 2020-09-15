use anyhow::Result;
use std::{panic, process};

// These modules contain macros. They must be declared before the others.
#[rustfmt::skip] mod log;
#[rustfmt::skip] mod util;

mod channels;
mod cmd;
mod events;
mod model;
mod proxy;
mod telnet;
mod ui;

use cmd::CmdArgs;
use model::Model;

fn main() -> Result<()> {
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        orig_hook(info);
        process::exit(1);
    }));

    let args = CmdArgs::get();
    let mut model = Model::new(
        args.proxy_host.as_str(),
        args.proxy_port,
        args.telnet_port,
        args.timeout,
    )?;

    model.start()?;

    Ok(())
}
