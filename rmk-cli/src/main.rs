use log::info;
use pretty_env_logger::env_logger::{Builder, Env};

use rmk_detection::watcher::create_watcher;
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::from_env(Env::new().default_filter_or("info")).init();

    let mut sig_term = signal(SignalKind::terminate())?;

    let (watcher, tx_stop, mut rx_device) = create_watcher()?;

    let watcher_handle = tokio::spawn(watcher);

    loop {
        select! {
            _ = signal::ctrl_c() => {
                info!("Received Ctrl-C, stopping");
                break;
            }
            _ = sig_term.recv() => {
                info!("Received SIGTERM, stopping");
                break;
            }
            e = rx_device.recv() => {
            info!("Received device event: {:?}", e);
            }
        }
    }

    tx_stop.send(()).await?;
    watcher_handle.await?;

    Ok(())
}
