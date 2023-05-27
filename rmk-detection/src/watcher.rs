use std::{future::Future, time::Duration};

use errors::RmkDetectionError;
use log::info;
use rusb::{Context, Device, HotplugBuilder, UsbContext};
use tokio::{
    select,
    sync::{broadcast, mpsc},
};

use crate::errors;

const VENDOR_ID: u16 = 0x04b3;
const PRODUCT_ID: u16 = 0x4010;

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    Connection(u8),
    Disconnection(u8),
}

pub struct RmkHotPlug {
    tx: broadcast::Sender<DeviceEvent>,
}

impl RmkHotPlug {
    pub fn new(tx: broadcast::Sender<DeviceEvent>) -> Self {
        RmkHotPlug { tx }
    }
}

impl rusb::Hotplug<Context> for RmkHotPlug {
    fn device_arrived(&mut self, device: Device<Context>) {
        info!("device arrived {:?}", device);

        match self.tx.send(DeviceEvent::Connection(device.bus_number())) {
            Ok(_) => (),
            Err(e) => {
                info!("error sending device event: {:?}", e);
            }
        }
    }

    fn device_left(&mut self, device: Device<Context>) {
        info!("device left {:?}", device);
        match self
            .tx
            .send(DeviceEvent::Disconnection(device.bus_number()))
        {
            Ok(_) => (),
            Err(e) => {
                info!("error sending device event: {:?}", e);
            }
        }
    }
}

pub fn create_watcher() -> Result<
    (
        impl Future<Output = ()>,
        mpsc::Sender<()>,
        broadcast::Receiver<DeviceEvent>,
    ),
    RmkDetectionError,
> {
    info!("starting hotplug");

    if rusb::has_hotplug() {
        let (tx_stop, mut rx_stop) = mpsc::channel::<()>(1);
        let (tx_device, rx_device) = broadcast::channel(32);

        let watch = async move {
            let context = Context::new().unwrap();

            let watcher = RmkHotPlug::new(tx_device);

            let _registration = HotplugBuilder::new()
                .enumerate(true)
                .vendor_id(VENDOR_ID)
                .product_id(PRODUCT_ID)
                .register(&context, Box::new(watcher))
                .unwrap();

            loop {
                let event = async { context.handle_events(Some(Duration::from_secs(1))) };

                select! {
                    _ = event => (),
                    _ = rx_stop.recv() => {
                        info!("stopping hotplug");
                        break
                    },
                }
            }
        };

        Ok((watch, tx_stop, rx_device))
    } else {
        Err(RmkDetectionError::HotPlugUnsupported)
    }
}
