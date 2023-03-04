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
        name: &str,
    ) -> Result<(Duration, FileAttr, u64), AsyncFilesystemError>;

    async fn readdir(
        &mut self,
        ino: u64,
        fh: u64,
        offset: i64,
    ) -> Result<Vec<(u64, i64, FileType, String)>, AsyncFilesystemError>;

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
        match self.rt.block_on(self.fs.getattr(ino)) {
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
        let r = name
            .to_str()
            .ok_or_else(|| AsyncFilesystemError::InvalidUtf8(name.to_os_string()))
            .and_then(|n| self.rt.block_on(self.fs.lookup(parent, n)));

        match r {
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
        match self.rt.block_on(self.fs.readdir(ino, fh, offset)) {
            Ok(entries) => {
                for (ino, offset, kind, name) in entries {
                    if reply.add(ino, offset, kind, name) {
                        break;
                    }
                }

                reply.ok();
            }
            Err(_) => reply.error(libc::ENOENT),
        }
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
            .block_on(self.fs.read(ino, fh, offset, size, flags, lock_owner))
        {
            Ok(data) => reply.data(data),
            Err(_) => reply.error(libc::ENOENT),
        }
    }
}
