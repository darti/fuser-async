use thiserror::Error;

#[derive(Error, Debug)]
pub enum RmkDetectionError {
    #[error("libusb hotplug api unsupported")]
    HotPlugUnsupported,

    #[error(transparent)]
    LibUsbError(#[from] rusb::Error),

    #[error(transparent)]
    OpenDALError(#[from] opendal::Error),

    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}
