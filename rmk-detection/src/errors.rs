use thiserror::Error;

#[derive(Error, Debug)]
pub enum RmkDetectionError {
    #[error("libusb hotplug api unsupported")]
    HotPlugUnsupported,

    #[error(transparent)]
    LibUsbError(#[from] rusb::Error),
}
