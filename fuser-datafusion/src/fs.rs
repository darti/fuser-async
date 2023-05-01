use std::time::Duration;

use async_trait::async_trait;
use datafusion::prelude::*;

use fuser_async::{
    async_filesystem::AsyncFilesystem,
    fuser::{FileAttr, FileType},
};
use itertools::izip;
use log::debug;

use crate::{
    conversion::{to_file_attr, BatchesIterators},
    errors::DatafusionFsError,
};

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
        debug!("getattr({})", ino);

        let query = format!(
            r#"SELECT 
            metadata.ino, 
            type,
            size
            FROM metadata 
            LEFT JOIN content ON metadata.ino = content.ino 
            WHERE metadata.ino = {} LIMIT 1"#,
            ino,
        );

        let batches = self.ctx.sql(&query).await?.collect().await?;

        let r = to_file_attr(batches);

        r
    }

    async fn lookup(
        &self,
        parent: u64,
        name: &str,
    ) -> Result<(Duration, FileAttr, u64), Self::Error> {
        debug!("lookup({}, {})", parent, name);

        let query = format!(
            "SELECT ino, type FROM {} WHERE name = '{}'  and parent_ino = {} LIMIT 1",
            METADATA_TABLE, name, parent
        );

        let batches = self.ctx.sql(&query).await?.collect().await?;

        let (ttl, attr) = to_file_attr(batches)?;

        Ok((ttl, attr, 0))
    }

    async fn readdir(
        &self,
        ino: u64,
        _fh: u64,
        offset: i64,
    ) -> Result<Vec<(u64, i64, FileType, String)>, Self::Error> {
        debug!("readdir({}, {})", ino, offset);

        let batches = self
            .ctx
            .table(METADATA_TABLE)
            .await?
            .filter(col("parent_ino").eq(lit(ino)))?
            .select_columns(&["ino", "name", "type"])?
            .sort(vec![col("ino").sort(true, true)])?
            .limit(offset as usize, None)?
            .collect()
            .await?;

        let r = izip!(batches.inos(0), batches.kinds(2), batches.names(1))
            .enumerate()
            .map(|(i, (ino, kind, name))| match (ino, kind, name) {
                (Some(ino), Some(kind), Some(name)) => {
                    Some((ino, i as i64 + 1, kind, name.to_owned()))
                }
                _ => None,
            })
            .flatten()
            .collect();

        Ok(r)
    }

    async fn read(
        &self,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock: Option<u64>,
    ) -> Result<Vec<u8>, Self::Error> {
        debug!(
            "read({}, {},  {}, {}, {:?})",
            ino, offset, size, flags, lock
        );

        let query = format!(
            "SELECT size, content FROM {} WHERE ino = {} LIMIT 1",
            CONTENT_TABLE, ino
        );

        self.ctx
            .sql(&query)
            .await?
            .collect()
            .await?
            .content(1)
            .flatten()
            .next()
            .map(|c| c.to_vec())
            .ok_or(DatafusionFsError::NotFound)
    }
}
