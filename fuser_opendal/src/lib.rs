use std::time::Duration;

use async_trait::async_trait;
use errors::OpendalFsError;
use fuser_async::{
    async_filesystem::AsyncFilesystem,
    fuser::{FileAttr, FileType},
};
use log::debug;
use opendal::Operator;

pub mod errors;

pub struct OpendalFs {
    operator: Operator,
}

impl OpendalFs {
    pub fn new(operator: Operator) -> Self {
        Self { operator }
    }
}

#[async_trait]
impl AsyncFilesystem for OpendalFs {
    type Error = OpendalFsError;

    async fn getattr(&self, ino: u64) -> Result<(Duration, FileAttr), Self::Error> {
        debug!("getattr(ino={})", ino);
        todo!()
    }

    async fn lookup(
        &self,
        parent: u64,
        name: &str,
    ) -> Result<(Duration, FileAttr, u64), Self::Error> {
        debug!("lookup(parent={}, name={})", parent, name);
        todo!()
    }

    async fn readdir(
        &self,
        ino: u64,
        fh: u64,
        offset: i64,
    ) -> Result<Vec<(u64, i64, FileType, String)>, Self::Error> {
        debug!("readdir(ino={}, fh={}, offset={})", ino, fh, offset);
        todo!()
    }

    async fn read(
        &self,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock: Option<u64>,
    ) -> Result<Vec<u8>, Self::Error> {
        debug!(
            "read(ino={}, fh={}, offset={}, size={}, flags={}, lock={})",
            ino,
            fh,
            offset,
            size,
            flags,
            lock.unwrap_or(0)
        );
        todo!()
    }
}
