//! In-memory key-value storage.

use super::types::{Command, Key, KeyRef, Value};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct Backend {
    data: HashMap<Key, Value>,
    commands: mpsc::Receiver<Command>,
}

#[derive(Debug, Clone)]
pub struct Store {
    commands: mpsc::Sender<Command>,
}

pub fn start() -> Store {
    let (tx, rx) = mpsc::channel(32);

    let backend = Backend {
        data: HashMap::new(),
        commands: rx,
    };

    tokio::spawn(backend.start());

    Store { commands: tx }
}

#[async_trait]
impl super::Store for Store {
    type Err = anyhow::Error;

    async fn get<'k>(&self, key: KeyRef<'k>) -> Result<Option<Value>, Self::Err> {
        let (tx, rx) = oneshot::channel();
        self.commands
            .send(Command::Get {
                key: key.to_owned(),
                cb: tx,
            })
            .await
            .context("unable to send get command")?;
        rx.await.context("unable to access result of get command")
    }

    async fn set(&mut self, key: Key, value: Value) -> Result<(), Self::Err> {
        self.commands
            .send(Command::Set {
                key: key.to_owned(),
                value,
            })
            .await
            .context("unable to send set command")
    }
}

impl Backend {
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
