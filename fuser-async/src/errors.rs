use std::{ffi::OsString, io};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AsyncFilesystemError {
    #[error("Mount error: {0}")]
    MountError(#[from] io::Error),

    #[error("Getattr error for ino {0}: {1}")]
    GetAttrError(u64, String),

    #[error("Getattr error for ino {0}: {1}")]
    ReadError(u64, String),

    #[error("invalid utf8: {0:?}")]
    InvalidUtf8(OsString),
}
