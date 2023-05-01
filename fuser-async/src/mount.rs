use std::{future::Future, path::Path};

use fuser::MountOption;
use log::info;
use tokio::runtime::Handle;

use crate::{
    async_filesystem::{AsyncFilesystem, AsyncFsImpl},
    errors::AsyncFilesystemError,
};

pub fn spawn_mount<'a, FS: AsyncFilesystem + Send + 'static + 'a, P: AsRef<Path>>(
    filesystem: FS,
    mountpoint: P,
    options: &[MountOption],
) -> Result<impl Future<Output = ()>, AsyncFilesystemError> {
    // check_option_conflicts(options)?;
    let afs = AsyncFsImpl::new(filesystem, Handle::current());

    let bs = fuser::spawn_mount2(afs, mountpoint, options)
        .map_err(|e| AsyncFilesystemError::MountError(e))?;

    let umount = async move {
        info!("Unmounting...");
        bs.join();
        info!("Unmounted");
    };

    Ok(umount)
}
