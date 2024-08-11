use std::io;
use std::path::Path;

use opendal::{services::Fs, Operator};
use opendal_mount::mount::Mounter;
use opendal_mount::{mount::FsMounter, NFSService, OpendalFs};
use snafu::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

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
}

impl RmkFs {
    #[tracing::instrument(name = "Init Rmk FS")]
    pub async fn new<P>(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
        cache_host: &str,
        root: &str,
        cache_mountpoint: P,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        info!("Mounting cache at {}", root);
        let builder = Fs::default().root(root);
        let cache = Operator::new(builder).context(CacheCreationSnafu)?.finish();

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

        info!("Mounting cache at {:?}", cache_mountpoint);
        FsMounter::mount(
            &addr.ip().to_string(),
            addr.port(),
            "",
            cache_mountpoint,
            false,
        )
        .await
        .context(CacheMountSnafu)?;

        Ok(Self { nfs_cache })
    }
}
