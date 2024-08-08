use opendal::{services::Fs, Operator};
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, span, Level};

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let mut sig_term = signal(SignalKind::terminate())?;

    let builder = Fs::default().root(".");
    let op = Operator::new(builder)?.finish();

    info!("Running, press Ctrl-C to stop");

    select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, stopping");

        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, stopping");

        }
    }

    info!("Clean exit");

    Ok(())
}
