//! Communication gateway meant to mediate access to storage.

use super::{
    codec::Codec,
    types::{Request, Response},
};
use crate::storage::Store;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::info;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

#[derive(Debug)]
pub struct StoreProtocol<T, S> {
    framed: Framed<T, Codec>,
    store: S,
}

impl<T, S> StoreProtocol<T, S>
where
    T: AsyncRead + AsyncWrite + Unpin,
    S: Store<Err = anyhow::Error>,
{
    pub fn new(conn: T, store: S) -> Self {
        Self {
            framed: Framed::new(conn, Codec::default()),
            store,
        }
    }

    pub async fn handle(mut self) -> Result<()> {
        while let Some(req) = self.framed.next().await {
            let res = self.process(req?).await?;
            self.framed.send(res).await?;
        }
        Ok(())
    }

    async fn process(&mut self, req: Request) -> Result<Response> {
        match req {
            Request::Get { key } => {
                info!("Get: key: {}", key);
                let value = self.get_from_store(&key).await?;
                Ok(Response::Get { key, value })
            }
            Request::Set { key, value } => {
                info!("Set: key: {} value: {}", key, value);
                self.set_into_store(key.clone(), value).await?;
                Ok(Response::Set { key })
            }
        }
    }

    async fn get_from_store(&mut self, key: &str) -> Result<Option<String>> {
        self.store.get(key).await
    }

    async fn set_into_store(&mut self, key: String, value: String) -> Result<()> {
        self.store.set(key, value).await
    }
}
