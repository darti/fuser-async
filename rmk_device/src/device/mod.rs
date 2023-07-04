use futures::TryStreamExt;

use opendal::{services, Metakey, Operator};

use crate::errors::RmkDetectionError;

pub struct RmkTablet {
    operator: Operator,
}

impl RmkTablet {
    pub fn new(
        endpoint: &str,
        user: &str,
        key_file: &str,
        base: &str,
    ) -> Result<Self, RmkDetectionError> {
        let mut builder = services::Sftp::default();
        builder
            .endpoint(endpoint)
            .user(user)
            .key(key_file)
            .root(base);

        let op = Operator::new(builder)?;

        Ok(RmkTablet {
            operator: op.finish(),
        })
    }

    // pub async fn scan(&self) -> Result<(), RmkDetectionError> {
    //     let mut ds = self.operator.list("./").await.unwrap();

    //     while let Some(de) = ds.try_next().await.unwrap() {
    //         let meta = self.operator.metadata(&de, Metakey::Mode).await.unwrap();

    //         if de.path().ends_with(".metadata") {
    //             let content = self.operator.read(de.path()).await?;
    //             let metadata: RmkMetadata = serde_json::from_slice(&content)?;

    //             info!("metadata: {:?}", metadata);
    //         } else if de.path().ends_with(".content") {
    //             let content = self.operator.read(de.path()).await?;
    //             info!("content: {:?}", content);
    //             let content: RmkContent = serde_json::from_slice(&content)?;

    //             info!("content: {:?}", content);
    //         }
    //     }

    //     Ok(())
    // }

    pub async fn scan(&self) -> Result<(), RmkDetectionError> {
        let mut ds = self.operator.list("./").await.unwrap();

        while let Some(de) = ds.try_next().await.unwrap() {
            let meta = self.operator.metadata(&de, Metakey::Mode).await.unwrap();

            let kind = if meta.is_dir() {
                "Directory"
            } else {
                "RegularFile"
            };

            let name = de.name();
            let size = meta.content_length();

            let mtime = meta.last_modified();

            // if de.path().ends_with(".metadata") {
            //     let content = self.operator.read(de.path()).await?;
            //     let metadata: RmkMetadata = serde_json::from_slice(&content)?;
            // } else if de.path().ends_with(".content") {
            //     let content = self.operator.read(de.path()).await?;

            //     let content: RmkContent = serde_json::from_slice(&content)?;
            // }
        }

        Ok(())
    }
}
