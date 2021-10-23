use anyhow::Result;
use log::info;
use structopt::StructOpt;
use tokio::net::TcpListener;
use toy_storage::{storage::inmemory, Server};

#[derive(StructOpt)]
struct Opts {
    #[structopt(short, long, default_value = "127.0.0.1:8080")]
    address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let opts = Opts::from_args();

    run_with(opts).await
}

async fn run_with(opts: Opts) -> Result<()> {
    info!("Listening at {}", opts.address);

    let listener = TcpListener::bind(opts.address).await?;

    let store = inmemory::start();

    Server::new(listener, store).start().await;

    Ok(())
}

fn init_logger() {
    env_logger::init()
}
