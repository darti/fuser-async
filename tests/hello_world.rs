use fuser_async::{async_filesystem::AsyncFilesystem, mount::spawn_mount};

struct SimpleFS {}

impl AsyncFilesystem for SimpleFS {}

#[tokio::test]
async fn test_mount() {
    let mountpoint = tempfile::tempdir().unwrap();

    let (umount_send, umount) =
        spawn_mount(SimpleFS {}, mountpoint, &[]).expect("Failed to mount filesystem");

    let um = tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        umount.await
    });

    tokio::select! {
        _ = um => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    // session.join();
}
