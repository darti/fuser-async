mod common;
mod version_5;
mod version_6;

use nom::bytes::complete::{tag, take};

use crate::{errors::RmkFormatError, format::Notebook};

use self::common::parse_int;

pub type ParseResult<'a, O> = Result<(&'a [u8], O), RmkFormatError>;

pub fn notebook(i: &[u8]) -> Result<Notebook, RmkFormatError> {
    let (i, version) = version(i)?;

    match version {
        5 => version_5::parse(i),
        6 => version_6::parse(i),
        _ => Err(RmkFormatError::UnsupportedVersion(version)),
    }
}

fn version(i: &[u8]) -> ParseResult<usize> {
    let (i, r) = take(43usize)(i).map_err(|e: nom::Err<(&[u8], nom::error::ErrorKind)>| {
        RmkFormatError::ParserError(e.map(|(ie, k)| (ie.to_vec(), k)))
    })?;

    let (r, _) = tag("reMarkable .lines file, version=")(r).map_err(
        |e: nom::Err<(&[u8], nom::error::ErrorKind)>| {
            RmkFormatError::ParserError(e.map(|(ie, k)| (ie.to_vec(), k)))
        },
    )?;
    let (_, version) = parse_int(r)?;

    Ok((i, version))
}
