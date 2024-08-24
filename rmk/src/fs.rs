use std::io;
use std::path::{Path, PathBuf};

use bytes::Bytes;
use opendal::{services::Fs, Operator};
use opendal_mount::mount::Mounter;
use opendal_mount::{mount::FsMounter, NFSService, OpendalFs};
use snafu::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

use crate::rmk_layer::RmkLayer;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error creating cache: {}", source))]
    CacheCreation { source: opendal::Error },

    #[snafu(display("Error mounting cache: {}", source))]
    CacheMount { source: io::Error },

    #[snafu(display("Error swith cache NFS: {}", source))]
    CacheNFS { source: io::Error },
}

#[derive(Debug, Clone)]
pub struct RmkFs {
    nfs_cache: NFSService<OpendalFs>,
    cache_mountpoint: PathBuf,
}

impl RmkFs {
    #[tracing::instrument(name = "Init Rmk FS", skip_all)]
    pub async fn new<P>(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
        cache_host: &str,
        root: &str,
        cache_mountpoint: P,
        volume_icon: Bytes,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        info!("Mounting cache at {}", root);
        let builder = Fs::default().root(root);
        let cache = Operator::new(builder)
            .context(CacheCreationSnafu)?
            .layer(RmkLayer::new())
            .finish();

        let nfs_cache = NFSService::new(
            OpendalFs::new(cache),
            "localhost:0",
            Some(cancellation_token.clone()),
            Some(task_tracker.clone()),
        )
        .await
        .context(CacheNFSSnafu)?;

        let nfs = nfs_cache.clone();
        let addr = nfs.local_addr();

        info!("Registered Cache NFS service , listening on {:?}", addr);

        let _h = task_tracker.spawn(async move {
            match nfs.handle().await {
                Ok(_) => info!("NFS service stopped"),
                Err(e) => error!("Error handling NFS service: {:?}", e),
            };
        });

        let cache_mountpoint_local = cache_mountpoint.as_ref().to_path_buf();

        info!("Mounting cache at {:?}", cache_mountpoint);
        FsMounter::mount(
            &addr.ip().to_string(),
            addr.port(),
            "",
            cache_mountpoint,
            true,
        )
        .await
        .context(CacheMountSnafu)?;

        Ok(Self {
            nfs_cache,
            cache_mountpoint: cache_mountpoint_local,
        })
    }
}

impl Drop for RmkFs {
    fn drop(&mut self) {
        info!("Unmounting cache");
        let _ = FsMounter::umount(self.cache_mountpoint.clone());
    }
}
