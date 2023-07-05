use log::info;

use opendal::{services::Fs, Operator};

use opendal_mount::{overlay::policy::OsFilesPolicy, serve, Overlay};
use rmk_device::config::SETTINGS;
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

const HOSTPORT: u32 = 12000;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    console_subscriber::init();

    #[cfg(not(feature = "tracing"))]
    pretty_env_logger::init();

    let mut remote_builder = Fs::default();

    remote_builder.root(&SETTINGS.config().local.base().display().to_string());

    let mut overlay_builder = Fs::default();
    overlay_builder.root(&SETTINGS.config().local.overlay().display().to_string());

    let mut builder = Overlay::default();
    builder
        .policy(OsFilesPolicy)
        .base_builder(remote_builder)
        .overlay_builder(overlay_builder);

    let opperator = Operator::new(builder)?.finish();

    let ipstr = format!("127.0.0.1:{HOSTPORT}");

    tokio::spawn(async move {
        info!("Serving on {}", ipstr);
        serve(&ipstr, opperator).await
    });

    let mut sig_term = signal(SignalKind::terminate())?;

    select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, sending unmount signals");
        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, sending unmount signal");
        }
    };

    Ok(())
}
