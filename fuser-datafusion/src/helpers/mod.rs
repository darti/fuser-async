use std::sync::Arc;

use base64::{decoded_len_estimate, prelude::*};
use datafusion::{
    arrow::array::{ArrayRef, UInt64Array},
    common::cast::as_string_array,
};

use crate::BinArray;

pub fn to_binary(args: &[ArrayRef]) -> datafusion::error::Result<ArrayRef> {
    let s = as_string_array(&args[0]).expect("cast failed");

    let mut buffer: Vec<u8> = vec![];

    match s.iter().next() {
        Some(Some(v)) => {
            buffer = vec![0; decoded_len_estimate(v.len())];
            let data = BASE64_STANDARD_NO_PAD
                .decode_slice(v, &mut buffer)
                .expect("decode failed");
            Some(data)
        }
        _ => None,
    };

    Ok(Arc::new(BinArray::from_vec(vec![buffer.as_slice()])) as ArrayRef)
}

pub fn binary_size(args: &[ArrayRef]) -> datafusion::error::Result<ArrayRef> {
    let s = args[0]
        .as_any()
        .downcast_ref::<BinArray>()
        .expect("cast failed");

    let array = s
        .iter()
        .map(|v| v.map(|v| v.len() as u64))
        .collect::<UInt64Array>();

    Ok(Arc::new(array) as ArrayRef)
}
