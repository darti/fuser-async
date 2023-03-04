use std::sync::Arc;

use datafusion::{
    arrow::{
        array::{ArrayRef, BinaryArray, LargeBinaryArray, UInt64Array},
        datatypes::DataType,
    },
    common::cast::{as_binary_array, as_string_array},
    logical_expr::Volatility,
    physical_plan::{functions::make_scalar_function, ColumnarValue},
    prelude::*,
};
use fuser_datafusion::{CONTENT_SCHEMA, CONTENT_TABLE, METADATA_SCHEMA, METADATA_TABLE};

fn to_binary(args: &[ArrayRef]) -> datafusion::error::Result<ArrayRef> {
    let s = as_string_array(&args[0]).expect("cast failed");

    let array = s
        .iter()
        .map(|v| v.map(|v| v.as_bytes()))
        .collect::<BinaryArray>();

    Ok(Arc::new(array) as ArrayRef)
}

fn binary_size(args: &[ArrayRef]) -> datafusion::error::Result<ArrayRef> {
    let s = as_binary_array(&args[0]).expect("cast failed");

    let array = s
        .iter()
        .map(|v| v.map(|v| v.len() as u64))
        .collect::<UInt64Array>();

    Ok(Arc::new(array) as ArrayRef)
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let ctx = SessionContext::new();

    ctx.register_udf(create_udf(
        "to_binary",
        vec![DataType::Utf8],
        Arc::new(DataType::Binary),
        Volatility::Immutable,
        make_scalar_function(to_binary),
    ));

    ctx.register_udf(create_udf(
        "binary_size",
        vec![DataType::Binary],
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
