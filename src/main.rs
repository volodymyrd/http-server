use crate::model::{Error, Handler, HttpMethod, HttpRequest, HttpResponse, Result};
use crate::server::Server;
use std::pin::Pin;
use tokio::net::TcpListener;

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

    let handler = RequestHandler;
    let handler = JsonContentType::new(handler);

    Server::new(listener).run(handler).await?;
    Ok(())
}

#[derive(Clone)]
struct RequestHandler;

impl Handler for RequestHandler {
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse>> + Send>>;

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        Box::pin(async move { handle_request(request).await })
    }
}

async fn handle_request(request: HttpRequest) -> Result<HttpResponse> {
    let response = match request.method_and_path() {
        (HttpMethod::Get, "/") => HttpResponse::ok("hello.html"),
        (_, _) => HttpResponse::not_found("404.html"),
    };

    Ok(response)
}

#[derive(Clone)]
struct JsonContentType<T> {
    inner_handler: T,
}

impl<T> JsonContentType<T>
where
    T: Handler,
{
    fn new(inner_handler: T) -> Self {
        Self { inner_handler }
    }
}

impl<T> Handler for JsonContentType<T>
where
    T: Handler + Clone + Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = crate::model::Result<HttpResponse>> + Send>>;

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let mut this = self.clone();

        Box::pin(async move {
            let response = match request.method_and_path() {
                (HttpMethod::Get, "/hi") => {
                    HttpResponse::ok_with_content_type("hi.json", "application/json")
                }
                (_, _) => this.inner_handler.call(request).await?,
            };

            Ok(response)
        })
    }
}
