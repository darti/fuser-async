use datafusion::prelude::*;
use fuser_datafusion::{helpers::create_context, CONTENT_TABLE, METADATA_SCHEMA, METADATA_TABLE};

use pretty_env_logger::env_logger::{Builder, Env};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::new().default_filter_or("info")).init();

    let ctx = create_context();

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
