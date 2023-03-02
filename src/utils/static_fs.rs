use crate::async_filesystem::AsyncFilesystem;

pub struct StaticFs {}

impl StaticFs {
    pub fn new() -> Self {
        Self {}
    }
}

// impl AsyncFilesystem for StaticFs {}
