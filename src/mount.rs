use std::{io, path::Path};

use fuser::{BackgroundSession, MountOption};

use crate::async_filesystem::{AsyncFilesystem, AsyncFsImpl};

pub fn mount<FS: AsyncFilesystem, P: AsRef<Path>>(
    filesystem: FS,
    mountpoint: P,
    options: &[MountOption],
) -> Result<(), io::Error> {
    let afs = AsyncFsImpl::new(filesystem);
    fuser::mount2(afs, mountpoint, options)
}

pub fn spawn_mount<'a, FS: AsyncFilesystem + Send + 'static + 'a, P: AsRef<Path>>(
    filesystem: FS,
    mountpoint: P,
    options: &[MountOption],
) -> Result<BackgroundSession, io::Error> {
    let afs = AsyncFsImpl::new(filesystem);
    fuser::spawn_mount2(afs, mountpoint, options)
}
