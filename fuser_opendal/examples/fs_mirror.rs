use std::env;

use anyhow::anyhow;
use fuser_async::mount::spawn_mount;
use fuser_opendal::OpendalFs;
use log::info;
use opendal::{services::Fs, Operator};
use pretty_env_logger::env_logger::{Builder, Env};
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::new().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <root>", args[0]);
        return Err(anyhow!("Usage: {} <root>", args[0]));
    }

    let mut builder = Fs::default();
    builder.root(&args[1]);

    let fs = OpendalFs::new(Operator::new(builder)?.finish());

    let mountpoint = tempfile::tempdir().unwrap();

    let umount = spawn_mount(fs, mountpoint, &[]).expect("Failed to mount filesystem");

    let mut sig_term = signal(SignalKind::terminate())?;

    select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, sending unmount signals");
        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, sending unmount signal");
        }
    };

    umount.await;

    Ok(())
}
