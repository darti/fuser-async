use std::{env, fs};

use log::info;
use rmk_format::notebook;

pub fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let path = env::args()
        .next()
        .expect("expected path to rm file as first argument");

    let bytes = fs::read(path)?;

    let notebook = notebook(&bytes)?;

    info!("{:#?}", notebook);

    Ok(())
}
