use core::str;
use std::future::Future;
use std::sync::Arc;
use std::vec::IntoIter;

use crate::table::{Entry as RmkEntry, InodeTable};
use opendal::raw::*;
use opendal::*;

pub struct RmkLayer {}

impl RmkLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<A: Access> Layer<A> for RmkLayer {
    type LayeredAccess = RmkAccessor<A>;

    fn layer(&self, inner: A) -> Self::LayeredAccess {
        RmkAccessor {
            inner,
            table: Arc::new(InodeTable::new()),
        }
    }
}

#[derive(Debug)]
pub struct RmkAccessor<A: Access> {
    inner: A,
    table: Arc<InodeTable>,
}

impl<A: Access> RmkAccessor<A> {
    fn ensure_scanned(&self) {}
}

impl<A: Access> LayeredAccess for RmkAccessor<A> {
    type Inner = A;
    type Reader = A::Reader;
    type BlockingReader = A::BlockingReader;
    type Writer = A::Writer;
    type BlockingWriter = A::BlockingWriter;
    type Lister = RmkLister;
    type BlockingLister = A::BlockingLister;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        self.inner.read(path, args).await
    }

    fn blocking_read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::BlockingReader)> {
        self.inner.blocking_read(path, args)
    }

    async fn write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        self.inner.write(path, args).await
    }

    fn blocking_write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::BlockingWriter)> {
        self.inner.blocking_write(path, args)
    }

    fn stat(&self, path: &str, args: OpStat) -> impl Future<Output = Result<RpStat>> + MaybeSend {
        self.inner().stat(path, args)
    }

    fn blocking_stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        self.inner().blocking_stat(path, args)
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Lister)> {
        self.table.scan(self.inner(), false).await.map_err(|e| {
            opendal::Error::new(ErrorKind::Unexpected, format!("Failed to scan: {}", e))
        })?;

        let normalized_path = normalize_path(path);

        let entries = self.table.list(&normalized_path).await.map_err(|e| {
            opendal::Error::new(
                ErrorKind::Unexpected,
                format!("Failed to list path {}: {}", path, e),
            )
        })?;

        Ok((RpList::default(), RmkLister::new(entries)))
    }

    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingLister)> {
        self.inner.blocking_list(path, args)
    }
}

pub struct RmkReader {}

pub struct RmkLister {
    entries: IntoIter<RmkEntry>,
}

impl RmkLister {
    pub fn new(entries: Vec<RmkEntry>) -> Self {
        Self {
            entries: entries.into_iter(),
        }
    }

    fn inner_next(&mut self) -> Option<oio::Entry> {
        self.entries.next().map(|entry| {
            let mode = if entry.meta.is_dir() {
                EntryMode::DIR
            } else {
                EntryMode::FILE
            };

            let name = format!(
                "{}{}",
                entry.meta.visible_name,
                if entry.meta.is_dir() { "/" } else { ".rmk" }
            );

            let meta = Metadata::new(mode);
            oio::Entry::with(name, meta)
        })
    }
}

impl oio::List for RmkLister {
    async fn next(&mut self) -> Result<Option<oio::Entry>> {
        Ok(self.inner_next())
    }
}

impl oio::BlockingList for RmkLister {
    fn next(&mut self) -> Result<Option<oio::Entry>> {
        Ok(self.inner_next())
    }
}
