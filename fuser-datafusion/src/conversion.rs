use std::time::{Duration, UNIX_EPOCH};

use datafusion::arrow::{
    array::{StringArray, UInt64Array},
    record_batch::RecordBatch,
};

use fuser_async::fuser::{FileAttr, FileType};
use itertools::izip;
use log::info;

use crate::{errors::DatafusionFsError, BinArray, BINARY_TYPE};

pub trait BatchesIterators {
    fn inos(&self, column: usize) -> Box<dyn Iterator<Item = Option<u64>> + '_>;
    fn kinds(&self, column: usize) -> Box<dyn Iterator<Item = Option<FileType>> + '_>;

    fn names(&self, column: usize) -> Box<dyn Iterator<Item = Option<&str>> + '_>;

    fn content(&self, column: usize) -> Box<dyn Iterator<Item = Option<&[u8]>> + '_>;
}

impl BatchesIterators for Vec<RecordBatch> {
    fn inos(&self, column: usize) -> Box<dyn Iterator<Item = Option<u64>> + '_> {
        let r = self
            .iter()
            .flat_map(move |batch| batch.column(column).as_any().downcast_ref::<UInt64Array>())
            .flat_map(|array| array.iter());

        Box::new(r)
    }

    fn kinds(&self, column: usize) -> Box<dyn Iterator<Item = Option<FileType>> + '_> {
        let r = self
            .iter()
            .flat_map(move |batch| batch.column(column).as_any().downcast_ref::<StringArray>())
            .flat_map(|array| array.iter())
            .map(|s| match s {
                Some("Directory") => Some(FileType::Directory),
                Some("RegularFile") => Some(FileType::RegularFile),
                Some("Symlink") => Some(FileType::Symlink),
                Some("Socket") => Some(FileType::Socket),
                Some("CharDevice") => Some(FileType::CharDevice),
                Some("BlockDevice") => Some(FileType::BlockDevice),
                Some("NamePipe") => Some(FileType::NamedPipe),
                _ => None,
            });

        Box::new(r)
    }

    fn names(&self, column: usize) -> Box<dyn Iterator<Item = Option<&str>> + '_> {
        let r = self
            .iter()
            .flat_map(move |batch| batch.column(column).as_any().downcast_ref::<StringArray>())
            .flat_map(|array| array.iter());

        Box::new(r)
    }

    fn content(&self, column: usize) -> Box<dyn Iterator<Item = Option<&[u8]>> + '_> {
        let r = self
            .iter()
            .flat_map(move |batch| batch.column(column).as_any().downcast_ref::<BinArray>())
            .flat_map(|array| array.iter());

        Box::new(r)
    }
}

fn parse_file_type(s: &str) -> Option<FileType> {
    match s {
        "Directory" => Some(FileType::Directory),
        "RegularFile" => Some(FileType::RegularFile),
        "Symlink" => Some(FileType::Symlink),
        "Socket" => Some(FileType::Socket),
        "CharDevice" => Some(FileType::CharDevice),
        "BlockDevice" => Some(FileType::BlockDevice),
        "NamePipe" => Some(FileType::NamedPipe),
        _ => None,
    }
}

pub fn to_file_attr(batches: Vec<RecordBatch>) -> Result<(Duration, FileAttr), DatafusionFsError> {
    for batch in batches {
        let inos = batch.column(0).as_any().downcast_ref::<UInt64Array>();
        let kinds = batch.column(1).as_any().downcast_ref::<StringArray>();
        let sizes = batch.column(0).as_any().downcast_ref::<UInt64Array>();

        if let (Some(inos), Some(kinds), Some(sizes)) = (inos, kinds, sizes) {
            for (ino, kind, size) in izip!(inos, kinds, sizes) {
                if let (Some(ino), Some(kind)) = (ino, kind.and_then(parse_file_type)) {
                    let attr = FileAttr {
                        ino: ino,
                        size: size.unwrap_or(0),
                        blocks: 1,
                        atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                        mtime: UNIX_EPOCH,
                        ctime: UNIX_EPOCH,
                        crtime: UNIX_EPOCH,
                        kind: kind,
                        perm: 0o755,
                        nlink: 2,
                        uid: 501,
                        gid: 20,
                        rdev: 0,
                        flags: 0,
                        blksize: 512,
                    };

                    return Ok((Duration::from_secs(3600), attr));
                }
            }
        }
    }
    return Err(DatafusionFsError::NotFound);
}
