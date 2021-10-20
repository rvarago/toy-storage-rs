use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

pub struct Store {
    data: HashMap<Key, Value>,
    queries: mpsc::Receiver<Query>,
}

#[derive(Debug)]
pub enum Query {
    Get {
        key: Key,
        response: oneshot::Sender<Option<Value>>,
    },
    Set {
        key: Key,
        value: Value,
    },
}

pub type Key = String;
pub type KeyRef<'a> = &'a str;
pub type Value = String;

pub type Sender = mpsc::Sender<Query>;

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
                Query::Get { key, response } => {
                    let value = self.data.get(&key).map(Value::clone);
                    let _ = response.send(value);
                }
                Query::Set { key, value } => {
                    self.data.insert(key, value);
                }
            }
        }
    }
}

pub async fn get(key: KeyRef<'_>, store_tx: &mut Sender) -> Result<Option<Value>> {
    let (tx, rx) = oneshot::channel();
    store_tx
        .send(Query::Get {
            key: key.to_owned(),
            response: tx,
        })
        .await?;
    rx.await.map_err(anyhow::Error::from)
}

pub async fn set(key: KeyRef<'_>, value: Value, store_tx: &mut Sender) -> Result<()> {
    store_tx
        .send(Query::Set {
            key: key.to_owned(),
            value,
        })
        .await
        .map_err(anyhow::Error::from)
}
