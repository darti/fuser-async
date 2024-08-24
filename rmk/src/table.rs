use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
    time::Instant,
};

use indextree::{Arena, NodeId};
use opendal::raw::{
    oio::{BlockingList, BlockingRead},
    Access, OpList, OpRead,
};

use rmk_format::metadata::RmkMetadata;
use snafu::{OptionExt, ResultExt, Snafu};

use tracing::{info, warn};

const ROOT: &str = "ROOT";

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Upstream list error: {source}"))]
    UpstreamList { source: opendal::Error },

    #[snafu(display("Entries list error: {source}"))]
    EntriesList { source: opendal::Error },

    #[snafu(display("Table lock error: {lock_type}"))]
    Lock { lock_type: String },

    #[snafu(display("Invalid path segment {segment} in {path}"))]
    InvalidPath { segment: String, path: String },

    #[snafu(display("Segment {segment} of {path} doesn't map"))]
    SegmentPath { segment: String, path: String },

    #[snafu(display("Inexisting node {id}"))]
    Node { id: NodeId },
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub meta: RmkMetadata,
}

#[derive(Debug, Clone)]
pub struct InodeTable {
    inner: Arc<RwLock<InodeTableInner>>,
}

impl Default for InodeTable {
    fn default() -> Self {
        Self::new()
    }
}

impl InodeTable {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InodeTableInner::new())),
        }
    }

    pub fn get(&self, path: &str) -> Result<Entry, Error> {
        let table = self.inner.read().map_err(|_| {
            LockSnafu {
                lock_type: "read".to_string(),
            }
            .build()
        })?;

        table.entry(path)
    }

    pub fn list(&self, path: &str) -> Result<Vec<Entry>, Error> {
        let table = self.inner.read().map_err(|_| {
            LockSnafu {
                lock_type: "read".to_string(),
            }
            .build()
        })?;

        table.list(path)
    }

    pub fn scan<A: Access>(&self, accessor: &A, force: bool) -> Result<(), Error> {
        let read_table = self.inner.read().map_err(|_| {
            LockSnafu {
                lock_type: "read".to_string(),
            }
            .build()
        })?;

        if force || !read_table.scanned {
            info!(
                "Need to scan, dirty = {}, force = {}",
                read_table.scanned, force
            );

            drop(read_table);

            let mut write_table = self.inner.write().map_err(|_| {
                LockSnafu {
                    lock_type: "write".to_string(),
                }
                .build()
            })?;

            write_table.scan(accessor)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct InodeTableInner {
    inodes: Arena<Entry>,
    // id -> (inode, parent, visible_name)
    id_index: HashMap<String, (indextree::NodeId, Option<String>, String)>,
    scanned: bool,
}

impl InodeTableInner {
    pub fn new() -> Self {
        let capacity = 1024;
        Self {
            inodes: Arena::with_capacity(capacity),
            id_index: HashMap::with_capacity(capacity),
            scanned: false,
        }
    }

    pub fn list(&self, path: &str) -> Result<Vec<Entry>, Error> {
        let node = self.node(path)?;

        Ok(node
            .children(&self.inodes)
            .filter_map(|c| self.inodes.get(c))
            .map(|e| e.get().to_owned())
            .collect())
    }

    pub fn entry(&self, path: &str) -> Result<Entry, Error> {
        let node = self.node(path)?;
        Ok(self.inodes.get(node).unwrap().get().to_owned())
    }

    fn node(&self, path: &str) -> Result<NodeId, Error> {
        let parsed_path = Path::new(path);

        let (mut current, _, _) = self.id_index.get(ROOT).context(InvalidPathSnafu {
            path: path.to_string(),
            segment: ROOT.to_string(),
        })?;

        for c in parsed_path.components() {
            match c {
                std::path::Component::ParentDir => {}
                std::path::Component::Normal(n) => {
                    let name = n.to_str().context(InvalidPathSnafu {
                        segment: n.to_string_lossy().to_string(),
                        path: path.to_string(),
                    })?;

                    current = current
                        .children(&self.inodes)
                        .find_map(|c| {
                            self.inodes.get(c).and_then(|e| {
                                if e.get().meta.visible_name == name {
                                    Some(c)
                                } else {
                                    None
                                }
                            })
                        })
                        .context(SegmentPathSnafu {
                            path,
                            segment: name,
                        })?;
                }
                _ => (),
            }
        }

        Ok(current)
    }

    #[tracing::instrument(skip(self))]
    pub fn scan<A: Access>(&mut self, accessor: &A) -> Result<(), Error> {
        self.inodes.clear();
        self.id_index.clear();

        info!("Scanning");
        let start_time = Instant::now();

        let (_, mut lister) = accessor
            .blocking_list("/", OpList::default())
            .context(UpstreamListSnafu)?;

        let root = self.inodes.new_node(Entry {
            id: "/".to_string(),
            meta: RmkMetadata {
                visible_name: "/".to_string(),
                ..Default::default()
            },
        });

        self.id_index
            .insert(ROOT.to_owned(), (root, None, "/".to_string()));

        // Read all metadata files
        while let Some(entry) = lister.next().context(EntriesListSnafu)? {
            if entry.path().ends_with(".metadata") {
                if let Ok((_, mut reader)) = accessor.blocking_read(entry.path(), OpRead::default())
                {
                    if let Ok(buf) = reader.read() {
                        if let Ok(metadata) = serde_json::from_slice::<RmkMetadata>(&buf.to_vec()) {
                            let id = entry.path().trim_end_matches(".metadata").to_string();
                            let inode = self.inodes.new_node(Entry {
                                id: id.clone(),
                                meta: metadata.to_owned(),
                            });

                            self.id_index
                                .insert(id, (inode, metadata.parent, metadata.visible_name));
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

        // Build the tree
        for (id, (inode, parent, name)) in self.id_index.iter() {
            let parent = parent.clone().unwrap_or(ROOT.to_owned());

            if let Some((parent_inode, _, _)) = self.id_index.get(&parent) {
                // avoid ROOT -> ROOT
                if parent_inode != inode {
                    parent_inode.append(*inode, &mut self.inodes);
                }
            }
        }

        self.scanned = true;

        let elapsed = start_time.elapsed();
        info!("Scanned in {:?}", elapsed);

        Ok(())
    }
}
