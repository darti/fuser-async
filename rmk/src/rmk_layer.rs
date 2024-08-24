use core::str;
use std::future::Future;
use std::sync::Arc;
use std::vec::IntoIter;

use crate::table::{Entry as RmkEntry, InodeTable};
use opendal::raw::*;
use opendal::*;
use tracing::debug;

pub struct RmkLayer {}

impl Default for RmkLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl RmkLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<A: Access> Layer<A> for RmkLayer {
    type LayeredAccess = RmkAccessor<A>;

    fn layer(&self, inner: A) -> Self::LayeredAccess {
        let layer = RmkAccessor::new(inner);

        layer.unwrap()
    }
}

#[derive(Debug)]
pub struct RmkAccessor<A: Access> {
    inner: A,
    table: Arc<InodeTable>,
}

impl<A: Access> RmkAccessor<A> {
    #[tracing::instrument]
    fn new(inner: A) -> Result<Self> {
        let table = Arc::new(InodeTable::new());

        table.scan(&inner, false).map_err(|e| {
            opendal::Error::new(ErrorKind::Unexpected, format!("Failed to scan: {}", e))
        })?;

        Ok(Self { inner, table })
    }

    fn ensure_scanned(&self, force: bool) -> Result<(), Error> {
        self.table.scan(self.inner(), force).map_err(|e| {
            opendal::Error::new(ErrorKind::Unexpected, format!("Failed to scan: {}", e))
        })
    }

    fn list_common(&self, path: &str) -> Result<Vec<RmkEntry>, Error> {
        let normalized_path = normalize_path(path);

        self.table.list(&normalized_path).map_err(|e| {
            opendal::Error::new(
                ErrorKind::Unexpected,
                format!("Failed to list path {}: {}", path, e),
            )
        })
    }

    fn stat_common(&self, path: &str) -> Result<Metadata, Error> {
        let normalized_path = normalize_path(path);

        let entry = self
            .table
            .get(&normalized_path)
            .map_err(|e| {
                opendal::Error::new(
                    ErrorKind::Unexpected,
                    format!("Failed to get path {}: {}", path, e),
                )
            })
            .map_err(|e| {
                opendal::Error::new(ErrorKind::Unexpected, format!("Failed to get: {}", e))
            })?;

        let metadata = Metadata::new(if entry.meta.is_dir() || path.ends_with("/") {
            EntryMode::DIR
        } else {
            EntryMode::FILE
        })
        .with_content_length(0);

        Ok(metadata)
    }
}

impl<A: Access> LayeredAccess for RmkAccessor<A> {
    type Inner = A;
    type Reader = A::Reader;
    type BlockingReader = A::BlockingReader;
    type Writer = A::Writer;
    type BlockingWriter = A::BlockingWriter;
    type Lister = RmkLister;
    type BlockingLister = RmkLister;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn read(
        &self,
        path: &str,
        args: OpRead,
    ) -> impl Future<Output = Result<(RpRead, Self::Reader)>> + MaybeSend {
        self.inner.read(path, args)
    }

    fn blocking_read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::BlockingReader)> {
        self.inner.blocking_read(path, args)
    }

    fn write(
        &self,
        path: &str,
        args: OpWrite,
    ) -> impl Future<Output = Result<(RpWrite, Self::Writer)>> + MaybeSend {
        self.inner.write(path, args)
    }

    fn blocking_write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::BlockingWriter)> {
        self.inner.blocking_write(path, args)
    }

    fn stat(&self, path: &str, args: OpStat) -> impl Future<Output = Result<RpStat>> + MaybeSend {
        async {
            let metadata = self.stat_common(path)?;

            Ok(RpStat::new(metadata))
        }
    }

    fn blocking_stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        let metadata = self.stat_common(path)?;

        Ok(RpStat::new(metadata))
    }

    fn list(
        &self,
        path: &str,
        args: OpList,
    ) -> impl Future<Output = Result<(RpList, Self::Lister)>> + MaybeSend {
        async {
            self.ensure_scanned(false)?;

            let entries = self.list_common(path)?;

            Ok((RpList::default(), RmkLister::new(path, entries)))
        }
    }

    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingLister)> {
        self.ensure_scanned(false)?;

        let entries = self.list_common(path)?;

        Ok((RpList::default(), RmkLister::new(path, entries)))
    }
}

pub struct RmkReader {}

pub struct RmkLister {
    entries: IntoIter<RmkEntry>,
    root: String,
}

impl RmkLister {
    pub fn new(root: &str, entries: Vec<RmkEntry>) -> Self {
        Self {
            entries: entries.into_iter(),
            root: root.to_string(),
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
                "{}/{}{}",
                self.root,
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
