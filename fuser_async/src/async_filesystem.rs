use std::{fmt, time::Duration};

use async_trait::async_trait;
use fuser::{FileAttr, FileType, Filesystem};
use log::{debug, error};

use tokio::runtime::Handle;

#[derive(Debug)]
pub struct Attr {
    pub ttl: Duration,
    pub attr: FileAttr,
}

#[derive(Debug)]
pub struct Lookup {
    pub ttl: Duration,
    pub attr: FileAttr,
    pub generation: u64,
}

#[derive(Debug)]
pub struct ReadDir {
    pub ino: u64,
    pub offset: i64,
    pub file_type: FileType,
    pub name: String,
}

#[async_trait]
pub trait AsyncFilesystem {
    type Error: fmt::Debug;
    async fn getattr(&self, ino: u64) -> Result<Attr, Self::Error>;

    async fn lookup(&self, parent: u64, name: &str) -> Result<Lookup, Self::Error>;

    async fn readdir(&self, ino: u64, fh: u64, offset: i64) -> Result<Vec<ReadDir>, Self::Error>;

    async fn read(
        &self,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock: Option<u64>,
    ) -> Result<Vec<u8>, Self::Error>;
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
            Ok(Attr { ttl, attr }) => {
                debug!("getattr({}) = {:?}", ino, attr);
                reply.attr(&ttl, &attr)
            }
            Err(e) => {
                error!("getattr({}) failed: {:?}", ino, e);
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
            .map(|n| self.rt.block_on(self.fs.lookup(parent, n)));

        debug!("lookup({:?}) = {:?}", name, r);

        match r {
            Some(Ok(Lookup {
                ttl,
                attr,
                generation,
            })) => reply.entry(&ttl, &attr, generation),

            Some(Err(e)) => {
                error!("lookup({:?}) failed: {:?}", name, e);
                reply.error(libc::ENOENT);
            }
            None => reply.error(libc::ENOENT),
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
                debug!("readdir({}) = {:?}", ino, entries);
                for r in entries {
                    if reply.add(r.ino, r.offset, r.file_type, r.name) {
                        break;
                    }
                }

                reply.ok();
            }
            Err(e) => {
                error!("readdir({}) failed: {:?}", ino, e);
                reply.error(libc::ENOENT)
            }
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
            Ok(data) => reply.data(&data),
            Err(e) => {
                error!("read({}) failed: {:?}", ino, e);
                reply.error(libc::ENOENT);
            }
        }
    }
}
