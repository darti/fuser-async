use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AsyncFilesystemError {
    #[error("Mount error: {0}")]
    MountError(#[from] io::Error),

    #[error("Getattr error for ino {0}: {1}")]
    GetAttrError(u64, String),
}
