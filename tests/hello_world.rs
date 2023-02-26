use fuser_async::{async_filesystem::AsyncFilesystem, mount::mount};

struct SimpleFS {}

impl AsyncFilesystem for SimpleFS {}

#[test]
fn test_mount() {
    let mountpoint = tempfile::tempdir().unwrap();

    mount(SimpleFS {}, mountpoint.path(), &[]).expect("Failed to mount filesystem");
}
