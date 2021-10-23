//! Network server meant to interact to service requests from clients.

use crate::{
    api::{framed, StoreService},
    storage::Store,
};
use log::{error, info};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
};

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
        info!("Received connection from {}", peer_addr);

        let service = self.new_service(conn);
        tokio::spawn(async move {
            match service.handle().await {
                Ok(_) => info!("Bye {}", peer_addr),
                Err(e) => error!("Oops from {}: {}", peer_addr, e),
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
