//! Communication gateway meant to mediate access to storage.

use super::types::{Request, Response};
use crate::storage::Store;
use anyhow::Result;
use futures::{Sink, SinkExt, Stream, StreamExt};
use tracing::info;

#[derive(Debug)]
pub struct StoreService<F, S> {
    frames: F,
    store: S,
}

impl<F, S> StoreService<F, S>
where
    F: Stream<Item = anyhow::Result<Request>> + Sink<Response, Error = anyhow::Error> + Unpin,
    S: Store<Err = anyhow::Error>,
{
    pub fn new(frames: F, store: S) -> Self {
        Self { frames, store }
    }

    pub async fn handle(mut self) -> Result<()> {
        while let Some(req) = self.frames.next().await {
            let res = self.process(req?).await?;
            self.frames.send(res).await?;
        }
        Ok(())
    }

    async fn process(&mut self, req: Request) -> Result<Response> {
        match req {
            Request::Get { key } => {
                info!("get: key: {}", key);
                let value = self.get_from_store(&key).await?;
                Ok(Response::Get { key, value })
            }
            Request::Set { key, value } => {
                info!("set: key: {} value: {}", key, value);
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
