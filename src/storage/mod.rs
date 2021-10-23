use self::types::{Key, KeyRef, Value};
use async_trait::async_trait;

pub mod inmemory;
pub mod types;

#[async_trait]
pub trait Store {
    type Err;

    async fn get<'k>(&self, key: KeyRef<'k>) -> Result<Option<Value>, Self::Err>;

    async fn set(&mut self, key: Key, value: Value) -> Result<(), Self::Err>;
}
