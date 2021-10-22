//! In-memory key-value storage.

use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct Store {
    data: HashMap<Key, Value>,
    queries: mpsc::Receiver<Command>,
}

#[derive(Debug)]
pub enum Command {
    Get {
        key: Key,
        cb: oneshot::Sender<Option<Value>>,
    },
    Set {
        key: Key,
        value: Value,
    },
}

pub type Key = String;
pub type KeyRef<'a> = &'a str;
pub type Value = String;

pub type Sender = mpsc::Sender<Command>;

impl Store {
    pub fn new() -> (Self, Sender) {
        let (tx, rx) = mpsc::channel(32);
        let store = Self {
            data: HashMap::new(),
            queries: rx,
        };
        (store, tx)
    }

    pub async fn start(mut self) {
        while let Some(query) = self.queries.recv().await {
            match query {
                Command::Get { key, cb } => {
                    let value = self.data.get(&key).map(Value::clone);
                    let _ = cb.send(value);
                }
                Command::Set { key, value } => {
                    self.data.insert(key, value);
                }
            }
        }
    }
}

pub async fn get(key: KeyRef<'_>, store_tx: &mut Sender) -> Result<Option<Value>> {
    let (tx, rx) = oneshot::channel();
    store_tx
        .send(Command::Get {
            key: key.to_owned(),
            cb: tx,
        })
        .await?;
    rx.await.map_err(anyhow::Error::from)
}

pub async fn set(key: KeyRef<'_>, value: Value, store_tx: &mut Sender) -> Result<()> {
    store_tx
        .send(Command::Set {
            key: key.to_owned(),
            value,
        })
        .await
        .map_err(anyhow::Error::from)
}
