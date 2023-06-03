use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpendalFsError {
    #[error(transparent)]
    OpendaError(#[from] opendal::Error),

    #[error("Inexisting node: {0}")]
    InexistingNode(u64),

    #[error("Lookup error: parent={0} name={1}")]
    LookupError(u64, String),
}
