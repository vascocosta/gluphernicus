mod gopher;
mod log;

use gopher::{Config, Server};
use std::sync::Arc;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let config = Config::from_args();
    let server = Arc::new(Server::new(config).await);

    if let Err(error) = server.run().await {
        eprintln!("Error running server: {}", error);
    }
}
