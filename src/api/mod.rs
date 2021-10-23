use self::codec::Codec;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

pub mod codec;
pub mod server;
pub mod service;
pub mod types;

pub use server::Server;

pub type StoreService<C, S> = service::StoreService<Framed<C, Codec>, S>;

pub fn framed<C: AsyncRead + AsyncWrite>(conn: C) -> Framed<C, Codec> {
    Framed::new(conn, Codec::default())
}
