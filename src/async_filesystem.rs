use std::{ffi::OsStr, time::Duration};

use async_trait::async_trait;
use fuser::{FileAttr, Filesystem};
use tokio::runtime::Handle;

use crate::errors::AsyncFilesystemError;

#[async_trait]
pub trait AsyncFilesystem {
    async fn getattr(&mut self, ino: u64) -> Result<(Duration, FileAttr), AsyncFilesystemError>;

    async fn lookup(
        &mut self,
        parent: u64,
        name: &OsStr,
    ) -> Result<(Duration, FileAttr, u64), AsyncFilesystemError>;
}

pub(crate) struct AsyncFsImpl<FS>
where
    FS: AsyncFilesystem,
{
    fs: FS,
    rt: Handle,
}

impl<FS> AsyncFsImpl<FS>
where
    FS: AsyncFilesystem,
{
    pub fn new(fs: FS, rt: Handle) -> Self {
        Self { fs, rt }
    }
}

impl<FS> Filesystem for AsyncFsImpl<FS>
where
    FS: AsyncFilesystem,
{
    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        match self.rt.block_on(async { self.fs.getattr(ino).await }) {
            Ok((ttl, attr)) => reply.attr(&ttl, &attr),
            Err(_e) => {
                reply.error(libc::ENOENT);
            }
        }
    }

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        match self
            .rt
            .block_on(async { self.fs.lookup(parent, name).await })
        {
            Ok((ttl, attr, generation)) => reply.entry(&ttl, &attr, generation),
            Err(_e) => {
                reply.error(libc::ENOENT);
            }
        }
    }
}
