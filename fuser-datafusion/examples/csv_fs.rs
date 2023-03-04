use datafusion::prelude::*;
use fuser_datafusion::{METADATA_SCHEMA, METADATA_TABLE};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let ctx = SessionContext::new();

    let options = CsvReadOptions::new().schema(&METADATA_SCHEMA);

    ctx.register_csv(METADATA_TABLE, "fuserdata/metadata.csv", options)
        .await?;

    let df = ctx.sql("SELECT * FROM metadata LIMIT 10").await?;

    df.show().await?;

    Ok(())
}
