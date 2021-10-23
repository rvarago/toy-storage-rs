//! Network server meant to interact to service requests from clients.

use crate::{
    api::{framed, StoreService},
    storage::Store,
};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
};
use tracing::{error, info, span, Level};

pub struct Server<S> {
    listener: TcpListener,
    store: S,
}

impl<S> Server<S>
where
    S: Store<Err = anyhow::Error> + Clone + Send + Sync + 'static,
{
    pub fn new(listener: TcpListener, store: S) -> Self {
        Self { listener, store }
    }

    pub async fn start(self) {
        while let Ok((conn, peer_addr)) = self.listener.accept().await {
            self.handle(conn, peer_addr)
        }
    }

    fn handle<C>(&self, conn: C, peer_addr: SocketAddr)
    where
        C: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let service = self.new_service(conn);

        tokio::spawn(async move {
            let span = span!(Level::INFO, "connection", peer_addr = %peer_addr);
            let _enter = span.enter();

            info!("serving new connection");

            match service.start().await {
                Ok(_) => info!("bye"),
                Err(e) => error!(reason = %e, "oops"),
            }
        });
    }

    fn new_service<C>(&self, conn: C) -> StoreService<C, S>
    where
        C: AsyncRead + AsyncWrite + Unpin,
    {
        StoreService::new(framed(conn), self.store.clone())
    }
}
