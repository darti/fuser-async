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
    type Lister = A::Lister;
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

        self.inner.list(path, args).await
    }

    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingLister)> {
        self.inner.blocking_list(path, args)
    }
}
