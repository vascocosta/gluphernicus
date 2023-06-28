mod gopher;
mod log;

use gopher::{Config, Server};
use std::sync::Arc;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let config = Config::from_args();
    let server = match Server::new(config).await {
        Ok(server) => server,
        Err(error) => {
            eprintln!("Error creating server: {}", error);

            return;
        }
    };
    let server = Arc::new(server);

    if let Err(error) = server.run().await {
        eprintln!("Error running server: {}", error);
    }
}
