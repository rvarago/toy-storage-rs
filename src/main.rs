use anyhow::Result;
use log::{error, info};
use structopt::StructOpt;
use tokio::net::TcpListener;
use toy_storage::{communication, store};

#[derive(StructOpt)]
struct Opts {
    #[structopt(short, long, default_value = "127.0.0.1:8080")]
    address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::from_args();

    info!("Listening at {}", opts.address);
    let listener = TcpListener::bind(opts.address).await?;

    let (store, store_tx) = store::Store::new();
    let server = Server::new(listener, store_tx);

    tokio::join!(server.start(), store.start());

    Ok(())
}

struct Server {
    listener: TcpListener,
    store_tx: store::Sender,
}

impl Server {
    fn new(listener: TcpListener, store_tx: store::Sender) -> Self {
        Self { listener, store_tx }
    }

    async fn start(self) {
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
