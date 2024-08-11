use opendal_mount::mount::FsMounter;
use opendal_mount::mount::Mounter;
use rmk::{fs::RmkFs, settings::SETTINGS};
use tempfile::tempdir;
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::info;

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let mut sig_term = signal(SignalKind::terminate())?;

    let cancellation_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    let cache_mountpoint = tempdir().unwrap();

    let fs = RmkFs::new(
        task_tracker,
        cancellation_token,
        "localhost:0",
        &SETTINGS.config().cache.root,
        cache_mountpoint.path(),
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

    info!("Unmounting NFS service");
    FsMounter::umount(cache_mountpoint).await?;

    info!("Clean exit");

    Ok(())
}
