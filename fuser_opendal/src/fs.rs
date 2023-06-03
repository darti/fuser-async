use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use async_trait::async_trait;
use errors::OpendalFsError;
use fuser_async::{
    async_filesystem::{AsyncFilesystem, Attr, Lookup, ReadDir},
    fuser::{FileAttr, FileType},
};
use log::debug;

use bimap::BiMap;

use futures::TryStreamExt;
use opendal::{Entry, Metadata, Metakey, Operator};
use parking_lot::RwLock;

use crate::errors;

pub struct OpendalFs {
    operator: Operator,
    inodes: Arc<RwLock<BiMap<u64, String>>>,
}

const TTL: Duration = Duration::from_secs(1);

impl OpendalFs {
    pub fn new(operator: Operator) -> Self {
        let mut inodes = BiMap::new();
        inodes.insert(1, "/".to_string());

        Self {
            operator,
            inodes: Arc::new(RwLock::new(inodes)),
        }
    }

    fn to_file_attr(&self, ino: u64, meta: &Metadata) -> Result<FileAttr, OpendalFsError> {
        let mtime = meta.last_modified().map(|t| t.into()).unwrap_or(UNIX_EPOCH);

        let kind = if meta.is_dir() {
            FileType::Directory
        } else {
            FileType::RegularFile
        };

        let perm = if meta.is_dir() { 0o755 } else { 0o644 };

        Ok(FileAttr {
            ino,
            size: meta.content_length(),
            blocks: 1,
            atime: mtime,
            mtime: mtime,
            ctime: mtime,
            crtime: mtime,
            kind,
            perm,
            nlink: 1,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        })
    }

    fn path(&self, ino: u64) -> Option<String> {
        self.inodes.read().get_by_left(&ino).cloned()
    }

    fn get_inode(&self, path: String) -> u64 {
        self.inodes
            .read()
            .get_by_right(&path)
            .copied()
            .unwrap_or_else(|| {
                let mut hasher = DefaultHasher::new();
                path.hash(&mut hasher);
                let ino = hasher.finish();

                self.inodes.write().insert(ino, path);

                ino
            })
    }
}

#[async_trait]
impl AsyncFilesystem for OpendalFs {
    type Error = OpendalFsError;

    async fn getattr(&self, ino: u64) -> Result<Attr, Self::Error> {
        debug!("getattr(ino={})", ino);

        let path = self.path(ino).ok_or(OpendalFsError::InexistingNode(ino))?;

        let entry = Entry::new(&path);
        let meta = self.operator.metadata(&entry, Metakey::Mode).await?;

        self.to_file_attr(ino, &meta)
            .map(|attr| Attr { ttl: TTL, attr })
    }

    async fn lookup(&self, parent: u64, name: &str) -> Result<Lookup, Self::Error> {
        debug!("lookup(parent={}, name={})", parent, name);

        Err(OpendalFsError::LookupError(parent, name.to_string()))
    }

    async fn readdir(&self, ino: u64, fh: u64, offset: i64) -> Result<Vec<ReadDir>, Self::Error> {
        debug!("readdir(ino={}, fh={}, offset={})", ino, fh, offset);

        let path = self.path(ino).ok_or(OpendalFsError::InexistingNode(ino))?;

        let mut ds = self.operator.list(&path).await?;

        let mut o = offset;

        while let Some(de) = ds.try_next().await.unwrap() {
            let meta = self.operator.metadata(&de, Metakey::Mode).await?;
            let entry_path = de.path().to_owned();

            let ino = self.get_inode(entry_path);
        }

        Ok(vec![])
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
