use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error, info};
use russh::{client, ChannelId};
use russh_keys::key;

use crate::errors::RmkDetectionError;

pub struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = RmkDetectionError;

    async fn check_server_key(
        self,
        server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        debug!("check_server_key: {:?}", server_public_key);
        Ok((self, true))
    }

    async fn data(
        self,
        channel: ChannelId,
        data: &[u8],
        session: client::Session,
    ) -> Result<(Self, client::Session), Self::Error> {
        info!(
            "data on channel {:?}: {:?}",
            channel,
            std::str::from_utf8(data)
        );
        Ok((self, session))
    }
}

pub async fn connect(
    ip: &str,
    port: u16,
    login: &str,
    password: &str,
) -> Result<(), RmkDetectionError> {
    let config: client::Config = russh::client::Config::default();
    let config = Arc::new(config);
    let sh = Client {};

    let mut session = russh::client::connect(config, (ip, port), sh).await?;

    if session.authenticate_password(login, password).await? {
        info!("Authenticated");
        let mut channel = session.channel_open_session().await?;

        let data = b"GET /les_affames.mkv HTTP/1.1\nUser-Agent: curl/7.68.0\nAccept: */*\nConnection: close\n\n";
        channel.data(&data[..]).await.unwrap();
    } else {
        error!("Failed to authenticate");
    }

    Ok(())
}
