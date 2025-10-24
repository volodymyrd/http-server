use crate::server::{Error, Result, Server};
use tokio::net::TcpListener;

#[cfg(test)]
mod integration_tests;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")
        .await
        .map_err(Error::Io)?;
    Server::new(listener).run().await?;
    Ok(())
}
