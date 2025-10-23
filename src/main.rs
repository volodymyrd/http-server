use crate::server::{Error, Result, Server};
use std::net::TcpListener;

mod server;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").map_err(Error::Io)?;
    Server::new(listener).run()?;
    Ok(())
}
