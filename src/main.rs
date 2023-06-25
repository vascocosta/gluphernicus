mod gopher;
mod log;

use crate::log::Logger;
use gopher::{Config, Server};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let config = Config::from_args();
    let logger = Arc::new(Mutex::new(Logger::new::<&str>(None).unwrap()));
    let server = Arc::new(Server::new(config, logger));

    if let Err(error) = server.run().await {
        eprintln!("Error running server: {}", error);
    }
}
