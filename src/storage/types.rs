use tokio::sync::oneshot;

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
