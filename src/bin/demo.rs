use fuser_async::{async_filesystem::AsyncFilesystem, mount::spawn_mount};
use log::info;
use pretty_env_logger::env_logger::{Builder, Env};

struct SimpleFS {}

impl AsyncFilesystem for SimpleFS {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::new().default_filter_or("info")).init();
    let mountpoint = tempfile::tempdir().unwrap();

    let (stop_sender, umount) =
        spawn_mount(SimpleFS {}, mountpoint, &[]).expect("Failed to mount filesystem");

    tokio::spawn(umount);

    let um = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        info!("Sending unmount signal");
        stop_sender.send(()).unwrap();
    });

    um.await?;

    Ok(())
}
