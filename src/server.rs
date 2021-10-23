//! Network server meant to interact to service requests from clients.

use crate::{api::service, storage::Store};
use log::{error, info};
use tokio::net::TcpListener;

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
        while let Ok((conn, peer)) = self.listener.accept().await {
            info!("Received connection from {}", peer);
            let protocol = service::StoreProtocol::new(conn, self.store.clone());
            tokio::spawn(async move {
                match protocol.handle().await {
                    Ok(_) => info!("Bye {}", peer),
                    Err(e) => error!("Oops from {}: {}", peer, e),
                }
            });
        }
    }
}
