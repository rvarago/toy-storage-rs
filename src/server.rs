//! Network server meant to interact to service requests from clients.

use crate::{api::communication, storage::inmemory};
use log::{error, info};
use tokio::net::TcpListener;

pub struct Server {
    listener: TcpListener,
    store_tx: inmemory::Sender,
}

impl Server {
    pub fn new(listener: TcpListener, store_tx: inmemory::Sender) -> Self {
        Self { listener, store_tx }
    }

    pub async fn start(self) {
        while let Ok((conn, peer)) = self.listener.accept().await {
            info!("Received connection from {}", peer);
            let protocol = communication::StoreProtocol::new(conn, self.store_tx.clone());
            tokio::spawn(async move {
                match protocol.handle().await {
                    Ok(_) => info!("Bye {}", peer),
                    Err(e) => error!("Oops from {}: {}", peer, e),
                }
            });
        }
    }
}
