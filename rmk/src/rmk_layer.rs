use core::str;
use std::sync::Arc;

use opendal::raw::*;
use opendal::*;

use crate::table::InodeTable;

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

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Lister)> {
        self.table.scan(self.inner(), false).await.map_err(|e| {
            opendal::Error::new(ErrorKind::Unexpected, format!("Failed to scan: {}", e))
        })?;

        let normalized_path = normalize_path(path);

        self.table.list(&normalized_path).await.map_err(|e| {
            opendal::Error::new(
                ErrorKind::Unexpected,
                format!("Failed to list path {}: {}", path, e),
            )
        })?;

        Ok((RpList::default(), RmkLister::new()))
    }

    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingLister)> {
        self.inner.blocking_list(path, args)
    }
}

pub struct RmkLister {}

impl RmkLister {
    pub fn new() -> Self {
        Self {}
    }

    fn inner_next(&mut self) -> Option<oio::Entry> {
        // self.idx.next().map(|v| {
        //     let mode = if v.ends_with('/') {
        //         EntryMode::DIR
        //     } else {
        //         EntryMode::FILE
        //     };
        //     let meta = Metadata::new(mode);
        //     oio::Entry::with(v, meta)
        // })
        //
        None
    }
}

impl oio::List for RmkLister {
    async fn next(&mut self) -> Result<Option<oio::Entry>> {
        todo!()
    }
}

impl oio::BlockingList for RmkLister {
    fn next(&mut self) -> Result<Option<oio::Entry>> {
        todo!()
    }
}
