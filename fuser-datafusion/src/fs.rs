use std::time::Duration;

use async_trait::async_trait;
use datafusion::prelude::*;

use fuser_async::{
    async_filesystem::AsyncFilesystem,
    fuser::{FileAttr, FileType},
};

use crate::errors::DatafusionFsError;

pub const METADATA_TABLE: &str = "metadata";
pub const CONTENT_TABLE: &str = "content";

pub struct DatafusionFs {
    ctx: SessionContext,
}

impl DatafusionFs {
    pub fn new(ctx: SessionContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl AsyncFilesystem for DatafusionFs {
    type Error = DatafusionFsError;
    async fn getattr(&self, ino: u64) -> Result<(Duration, FileAttr), Self::Error> {
        let query = format!(
            "SELECT ino FROM {} WHERE ino = {} LIMIT 1",
            METADATA_TABLE, ino
        );

        let df = self.ctx.sql(&query).await?.collect().await?;

        todo!()
    }

    async fn lookup(
        &self,
        parent: u64,
        name: &str,
    ) -> Result<(Duration, FileAttr, u64), Self::Error> {
        todo!()
    }

    async fn readdir(
        &self,
        ino: u64,
        fh: u64,
        offset: i64,
    ) -> Result<Vec<(u64, i64, FileType, String)>, Self::Error> {
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
    ) -> Result<&[u8], Self::Error> {
        todo!()
    }
}
