use datafusion::prelude::*;
use fuser_async::mount::spawn_mount;
use fuser_datafusion::{
    helpers::create_context, DatafusionFs, CONTENT_TABLE, METADATA_SCHEMA, METADATA_TABLE,
};

use log::info;

use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

async fn load_fs() -> datafusion::error::Result<SessionContext> {
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

    Ok(ctx)
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let ctx = load_fs().await?;
    let fs = DatafusionFs::new(ctx);
    let mountpoint = tempfile::tempdir().unwrap();

    info!("Mounting filesystem at {}", mountpoint.path().display());

    let (stop_sender, umount) =
        spawn_mount(fs, mountpoint, &[]).expect("Failed to mount filesystem");

    tokio::spawn(umount);

    let mut sig_term = signal(SignalKind::terminate())?;

    select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, sending unmount signals");
            stop_sender.send(()).unwrap();
        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, sending unmount signal");
            stop_sender.send(()).unwrap();
        }
    };

    Ok(())
}
