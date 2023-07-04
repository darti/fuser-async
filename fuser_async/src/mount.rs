use std::{future::Future, path::Path};

use fuser::{MountOption, Session};
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
) -> Result<
    (
        impl Future<Output = Result<(), AsyncFilesystemError>>,
        impl Future<Output = Result<(), AsyncFilesystemError>>,
    ),
    AsyncFilesystemError,
> {
    // check_option_conflicts(options)?;
    let afs = AsyncFsImpl::new(filesystem, Handle::current());

    let mut session = Session::new(afs, mountpoint.as_ref(), options)
        .map_err(|e| AsyncFilesystemError::MountError(e))?;

    let mut umount = session.unmount_callable();

    let umount = async move {
        info!("Unmounting...");
        umount
            .unmount()
            .map_err(|e| AsyncFilesystemError::MountError(e))?;
        info!("Unmounted");

        Ok(())
    };

    let mount = async move {
        let mut se = session;
        info!("Mounting...");
        se.run().map_err(|e| AsyncFilesystemError::MountError(e))?;

        Ok(())
    };

    Ok((mount, umount))
}
