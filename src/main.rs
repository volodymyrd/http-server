use crate::server::{Error, HttpRequest, HttpResponse, Result, Server};
use tokio::net::TcpListener;

#[cfg(test)]
mod integration_tests;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")
        .await
        .map_err(Error::Io)?;
    Server::new(listener).run(handle_request).await?;
    Ok(())
}

async fn handle_request(request: HttpRequest) -> Result<HttpResponse> {
    let response = if request.path() == "GET / HTTP/1.1\r\n" {
        HttpResponse::new("HTTP/1.1 200 OK", "hello.html")
    } else {
        HttpResponse::new("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    Ok(response)
}
