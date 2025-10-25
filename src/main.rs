use crate::model::{Error, HttpMethod, HttpRequest, HttpResponse, Result};
use tokio::net::TcpListener;
use crate::server::Server;

#[cfg(test)]
mod integration_tests;
mod model;
mod server;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")
        .await
        .map_err(Error::Io)?;
    Server::new(listener).run(handle_request).await?;
    Ok(())
}

async fn handle_request(request: HttpRequest) -> Result<HttpResponse> {
    let response = match request.method_and_path() {
        (HttpMethod::Get, "/") => HttpResponse::new("HTTP/1.1 200 OK", "hello.html"),
        (_, _) => HttpResponse::new("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    Ok(response)
}
