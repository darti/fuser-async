use crate::format::{
    LayerBuilderError, NotebookBuilderError, PointBuilderError, StrokeBuilderError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RmkFormatError {
    #[error("unsupported version: {0}")]
    UnsupportedVersion(usize),

    #[error("notebook build error")]
    NotebookBuildError(#[from] NotebookBuilderError),

    #[error("layer build error")]
    LayerBuildError(#[from] LayerBuilderError),

    #[error("stoke build error")]
    StrokeBuildError(#[from] StrokeBuilderError),

    #[error("point build error")]
    PointBuildError(#[from] PointBuilderError),

    #[error("parser error")]
    ParserError(#[from] nom::Err<(Vec<u8>, nom::error::ErrorKind)>),

    #[error("invalide version: {0}")]
    InvalidVersion(String),
}

// impl<'a> Into<RmkFormatError> for nom::Err<(&'a [u8], nom::error::ErrorKind)> {
//     fn into(self) -> RmkFormatError {
//         RmkFormatError::ParserError(self.map(|(ie, k)| (ie.to_vec(), k)))
//     }
// }
