use futures::TryStreamExt;
use log::info;
use opendal::{services, Metakey, Operator};
use rmk_format::{content::RmkContent, metadata::RmkMetadata};
use serde_json::Value;

use crate::errors::RmkDetectionError;

pub struct RmkTablet {
    operator: Operator,
}

impl RmkTablet {
    pub fn connect(
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

    pub async fn scan(&self) -> Result<(), RmkDetectionError> {
        let mut ds = self.operator.list("./").await.unwrap();

        while let Some(de) = ds.try_next().await.unwrap() {
            let meta = self.operator.metadata(&de, Metakey::Mode).await.unwrap();

            if de.path().ends_with(".metadata") {
                let content = self.operator.read(de.path()).await.unwrap();
                let metadata: RmkMetadata = serde_json::from_slice(&content)?;

                info!("metadata: {:?}", metadata);
            } else if de.path().ends_with(".content") {
                let content = self.operator.read(de.path()).await.unwrap();
                let content: RmkContent = serde_json::from_slice(&content)?;

                info!("content: {:?}", content);
            }
        }

        Ok(())
    }
}
