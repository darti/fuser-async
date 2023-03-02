use async_trait::async_trait;
use fuser::{FileAttr, FileType};
use fuser_async::{
    async_filesystem::AsyncFilesystem, errors::AsyncFilesystemError, mount::spawn_mount,
};
use log::info;
use pretty_env_logger::env_logger::{Builder, Env};
use std::time::{Duration, UNIX_EPOCH};
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

const TTL: Duration = Duration::from_secs(1);

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

const HELLO_TXT_CONTENT: &str = "Hello World!\n";

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

struct SimpleFS {}

#[async_trait]
impl AsyncFilesystem for SimpleFS {
    async fn getattr(&mut self, ino: u64) -> Result<(Duration, FileAttr), AsyncFilesystemError> {
        match ino {
            1 => Ok((TTL, HELLO_DIR_ATTR)),
            2 => Ok((TTL, HELLO_TXT_ATTR)),
            _ => Err(AsyncFilesystemError::GetAttrError(
                ino,
                "No such file or directory".to_string(),
            )),
        }
    }

    async fn lookup(
        &mut self,
        parent: u64,
        name: &std::ffi::OsStr,
    ) -> Result<(Duration, FileAttr, u64), AsyncFilesystemError> {
        match (parent, name.to_str()) {
            (1, Some("hello.txt")) => Ok((TTL, HELLO_TXT_ATTR, 2)),
            _ => Err(AsyncFilesystemError::GetAttrError(
                parent,
                "No such file or directory".to_string(),
            )),
        }
    }

    async fn readdir(
        &mut self,
        ino: u64,
        _fh: u64,
        offset: i64,
    ) -> Vec<(u64, i64, FileType, String)> {
        match ino {
            1 => {
                let mut entries = Vec::new();
                if offset == 0 {
                    entries.push((1, 1, FileType::Directory, ".".to_string()));
                }
                if offset <= 1 {
                    entries.push((1, 2, FileType::Directory, "..".to_string()));
                }
                if offset <= 2 {
                    entries.push((2, 3, FileType::RegularFile, "hello.txt".to_string()));
                }
                entries
            }
            _ => Vec::new(),
        }
    }

    async fn read(
        &mut self,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
    ) -> Result<&[u8], AsyncFilesystemError> {
        if ino == 2 {
            Ok(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..])
        } else {
            Err(AsyncFilesystemError::GetAttrError(
                ino,
                "No such file or directory".to_string(),
            ))
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::new().default_filter_or("info")).init();
    let mountpoint = tempfile::tempdir().unwrap();

    let (stop_sender, umount) =
        spawn_mount(SimpleFS {}, mountpoint, &[]).expect("Failed to mount filesystem");

    tokio::spawn(umount);

    let mut sig_term = signal(SignalKind::terminate())?;

    select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, sending unmount signals");
            stop_sender.send(()).unwrap();
        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, sending unmount signal");
            stop_sender.send(()).unwrap();
        }
    };

    Ok(())
}
