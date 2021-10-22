use anyhow::Result;
use log::info;
use structopt::StructOpt;
use tokio::net::TcpListener;
use toy_storage::{Server, Store};

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

    let (store, store_tx) = Store::new();
    let server = Server::new(listener, store_tx);

    tokio::join!(server.start(), store.start());

    Ok(())
}
