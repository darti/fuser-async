use std::str::from_utf8;

use nom::{
    bytes::complete::take_while1,
    character::is_digit,
    combinator::map,
    number::complete::{le_f32, le_u32},
};

use crate::errors::RmkFormatError;

use super::ParseResult;

#[inline]
pub fn parse_u32(i: &[u8]) -> ParseResult<u32> {
    le_u32(i).map_err(|e| RmkFormatError::ParserError(e.map(|(ie, k)| (ie.to_vec(), k))))
}

#[inline]
pub fn parse_f32(i: &[u8]) -> ParseResult<f32> {
    le_f32(i).map_err(|e| RmkFormatError::ParserError(e.map(|(ie, k)| (ie.to_vec(), k))))
}

pub fn parse_int<T>(i: &[u8]) -> ParseResult<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    map(take_while1(is_digit), |b| {
        from_utf8(b).unwrap().parse::<T>().unwrap()
    })(i)
    .map_err(|e: nom::Err<(&[u8], nom::error::ErrorKind)>| {
        RmkFormatError::ParserError(e.map(|(ie, k)| (ie.to_vec(), k)))
    })
}
