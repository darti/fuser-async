use std::{ffi::OsStr, time::Duration};

use async_trait::async_trait;
use fuser::{FileAttr, FileType, Filesystem};
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

    async fn readdir(
        &mut self,
        ino: u64,
        fh: u64,
        offset: i64,
    ) -> Vec<(u64, i64, FileType, String)>;

    async fn read(
        &mut self,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock: Option<u64>,
    ) -> Result<&[u8], AsyncFilesystemError>;
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

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        let entries = self
            .rt
            .block_on(async { self.fs.readdir(ino, fh, offset).await });

        for (ino, offset, kind, name) in entries {
            if reply.add(ino, offset, kind, name) {
                break;
            }
        }

        reply.ok();
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        match self
            .rt
            .block_on(async { self.fs.read(ino, fh, offset, size, flags, lock_owner).await })
        {
            Ok(data) => reply.data(data),
            Err(_) => reply.error(libc::ENOENT),
        }
    }
}
