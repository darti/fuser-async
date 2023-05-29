use log::{debug, error, info};
use pretty_env_logger::env_logger::{Builder, Env};

use rmk_detection::{
    config::SETTINGS,
    device::RmkTablet,
    watcher::{create_watcher, DeviceEvent},
};
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
                debug!("Received device event: {:?}", e);

                match e {
                    Ok(DeviceEvent::Connection(b)) => {
                        info!("Connected to device: {:?}", b);
                        let tablet = RmkTablet::connect(
                            &SETTINGS.config().device.endpoint,
                            &SETTINGS.config().device.user,
                            &SETTINGS.config().device.key_file,
                        &SETTINGS.remarkable().base)?;

                        tablet.scan().await;

                        // let r = client.execute("ls ").await?;
                        // info!("Received: {:?}", r);
                    }
                    Ok(DeviceEvent::Disconnection(b)) => {
                        info!("Disconnected from device: {:?}", b);
                    }
                    Err(e) => {
                        error!("Error receiving device event: {:?}", e);
                    }
                }
            }

        }
    }

    tx_stop.send(()).await?;
    watcher_handle.await?;

    Ok(())
}
