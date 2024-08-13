use rmk::{fs::RmkFs, settings::SETTINGS};
use tempfile::tempdir;
use tempfile::TempDir;
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::info;

use bytes::Bytes;

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let mut sig_term = signal(SignalKind::terminate())?;

    let cancellation_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    let cache_mountpoint = if let Some(prefix) = &SETTINGS.config().cache.mountpoint_prefix {
        TempDir::with_prefix_in("rmk-", prefix)
    } else {
        tempdir()
    }
    .unwrap();

    let _fs = RmkFs::new(
        task_tracker,
        cancellation_token,
        "localhost:0",
        &SETTINGS.config().cache.root,
        cache_mountpoint.path(),
        Bytes::from_static(include_bytes!("../assets/remarkable.icns")),
    )
    .await?;

    info!("Running, press Ctrl-C to stop");

    select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, stopping");

        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, stopping");

        }
    }

    info!("Clean exit");

    Ok(())
}
