//! In-memory key-value storage.

use super::types::{Command, Key, KeyRef, Value};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct Store {
    data: HashMap<Key, Value>,
    commands: mpsc::Receiver<Command>,
}

pub type Sender = mpsc::Sender<Command>;

impl Store {
    pub fn new() -> (Self, Sender) {
        let (tx, rx) = mpsc::channel(32);
        let store = Self {
            data: HashMap::new(),
            commands: rx,
        };
        (store, tx)
    }

    pub async fn start(mut self) {
        while let Some(command) = self.commands.recv().await {
            match command {
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
