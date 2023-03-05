use std::sync::Arc;

use base64::{decoded_len_estimate, prelude::*};
use datafusion::{
    arrow::{
        array::{ArrayData, ArrayDataBuilder, ArrayRef, UInt64Array},
        buffer::Buffer,
        datatypes::DataType,
    },
    common::cast::as_string_array,
    logical_expr::Volatility,
    physical_plan::functions::make_scalar_function,
    prelude::*,
};
use fuser_datafusion::{BinArray, BINARY_TYPE, CONTENT_TABLE, METADATA_SCHEMA, METADATA_TABLE};

use pretty_env_logger::env_logger::{Builder, Env};

fn to_binary(args: &[ArrayRef]) -> datafusion::error::Result<ArrayRef> {
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

fn binary_size(args: &[ArrayRef]) -> datafusion::error::Result<ArrayRef> {
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

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::new().default_filter_or("info")).init();

    let ctx = SessionContext::new();

    ctx.register_udf(create_udf(
        "to_binary",
        vec![DataType::Utf8],
        Arc::new(BINARY_TYPE),
        Volatility::Immutable,
        make_scalar_function(to_binary),
    ));

    ctx.register_udf(create_udf(
        "binary_size",
        vec![BINARY_TYPE],
        Arc::new(DataType::UInt64),
        Volatility::Immutable,
        make_scalar_function(binary_size),
    ));

    ctx.register_csv(
        METADATA_TABLE,
        "fuser-datafusion/examples/data/metadata.csv",
        CsvReadOptions::default().schema(&METADATA_SCHEMA),
    )
    .await?;

    let content = ctx
        .read_csv(
            "fuser-datafusion/examples/data/content.csv",
            CsvReadOptions::default(),
        )
        .await?;

    let to_binary = content.registry().udf("to_binary")?;
    let binary_size = content.registry().udf("binary_size")?;

    let content = content
        .with_column("content", to_binary.call(vec![col("content")]))?
        .with_column("size", binary_size.call(vec![col("content")]))?
        .select(vec![col("ino"), col("size"), col("content")])?;

    ctx.register_table(CONTENT_TABLE, content.into_view())?;

    ctx.sql("SELECT * FROM metadata LIMIT 10")
        .await?
        .show()
        .await?;

    ctx.sql("SELECT * FROM content LIMIT 10")
        .await?
        .show()
        .await?;

    Ok(())
}
