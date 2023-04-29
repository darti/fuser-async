use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatafusionFsError {
    #[error("Datafusion error: {0}")]
    DatafusionError(#[from] datafusion::error::DataFusionError),

    #[error("Not found")]
    NotFound,

    #[error("Not implemented")]
    NotImplemented,
}
