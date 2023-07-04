use std::env;

use anyhow::anyhow;
use fuser_async::{
    fuser::MountOption,
    mount::{self, spawn_mount},
};
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

use tokio::task;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    // Builder::from_env(Env::new().default_filter_or("info")).init();
    console_subscriber::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <root>", args[0]);
        return Err(anyhow!("Usage: {} <root>", args[0]));
    }

    let mut builder = Fs::default();
    builder.root(&args[1]);

    let fs = OpendalFs::new(Operator::new(builder)?.finish());
    let mountpoint = tempfile::tempdir()?;

    info!("Mounting filesystem at {}", mountpoint.path().display());
    let (mount, umount) = spawn_mount(
        fs,
        mountpoint,
        &[
            MountOption::AutoUnmount,
            MountOption::CUSTOM("-d".to_string()),
        ],
    )?;

    task::spawn_blocking(|| mount);
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
