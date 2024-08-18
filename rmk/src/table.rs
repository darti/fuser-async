use std::sync::Arc;

use indextree::Arena;
use opendal::raw::{
    oio::{List, Read},
    Access, OpList, OpRead,
};

use rmk_format::metadata::RmkMetadata;
use snafu::{ResultExt, Snafu};

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Upstream list error: {source}"))]
    UpstreamList { source: opendal::Error },
    #[snafu(display("Entries list error: {source}"))]
    EntriesList { source: opendal::Error },
    #[snafu(display("Table lock error"))]
    LockError {},
}

#[derive(Debug)]
struct Entry {
    pub id: String,
    pub meta: RmkMetadata,
}

#[derive(Debug, Clone)]
pub struct InodeTable {
    inner: Arc<RwLock<InodeTableInner>>,
}

impl InodeTable {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InodeTableInner::new())),
        }
    }

    pub async fn scan<A: Access>(&self, accessor: &A, force: bool) -> Result<(), Error> {
        let read_table = self.inner.read().await;

        if force || !read_table.scanned {
            info!(
                "Need to scan, dirty = {}, force = {}",
                read_table.scanned, force
            );

            drop(read_table);

            let mut write_table = self.inner.write().await;

            write_table.scan(accessor).await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct InodeTableInner {
    inodes: Arena<Entry>,
    scanned: bool,
}

impl InodeTableInner {
    pub fn new() -> Self {
        Self {
            inodes: Arena::with_capacity(1024),
            scanned: false,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn scan<A: Access>(&mut self, accessor: &A) -> Result<(), Error> {
        info!("Scanning");
        let (_, mut lister) = accessor
            .list("/", OpList::default())
            .await
            .context(UpstreamListSnafu)?;

        while let Some(entry) = lister.next().await.context(EntriesListSnafu)? {
            if entry.path().ends_with(".metadata") {
                if let Ok((_, mut reader)) = accessor.read(entry.path(), OpRead::default()).await {
                    if let Ok(buf) = reader.read_all().await {
                        if let Ok(metadata) = serde_json::from_slice::<RmkMetadata>(&buf.to_vec()) {
                            let id = entry.path().trim_end_matches(".metadata").to_string();
                            let inode = self.inodes.new_node(Entry { id, meta: metadata });
                            debug!("Scanned {}", entry.path());
                        } else {
                            warn!("Failed to deserialize metadata for {}", entry.path());
                        }
                    } else {
                        warn!("Failed to read metadata for {}", entry.path());
                    }
                } else {
                    warn!("Failed to open metadata for {}", entry.path());
                }
            }
        }

        self.scanned = true;

        Ok(())
    }
}
