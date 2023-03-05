use std::sync::Arc;

use datafusion::{
    arrow::datatypes::DataType, logical_expr::Volatility,
    physical_plan::functions::make_scalar_function, prelude::*,
};
use fuser_datafusion::{
    helpers::{binary_size, to_binary},
    BINARY_TYPE, CONTENT_TABLE, METADATA_SCHEMA, METADATA_TABLE,
};

use pretty_env_logger::env_logger::{Builder, Env};

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
