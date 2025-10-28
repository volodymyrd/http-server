use crate::model::{Error, Handler, HttpMethod, HttpRequest, HttpResponse, Result};
use crate::server::Server;
use std::future::Future;
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

impl Handler<HttpRequest> for RequestHandler {
    type Response = HttpResponse;
    type Error = Error;
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

impl<T> JsonContentType<T> {
    fn new(inner_handler: T) -> Self {
        Self { inner_handler }
    }
}

impl<T> Handler<HttpRequest> for JsonContentType<T>
where
    T: Handler<HttpRequest, Response = HttpResponse, Error = Error> + Clone + Send + 'static,
    <T as Handler<HttpRequest>>::Future: Send,
{
    type Response = HttpResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse>> + Send>>;

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let mut this = self.clone();

        Box::pin(async move {
            match request.method_and_path() {
                (HttpMethod::Get, "/hi") => Ok(HttpResponse::ok_with_content_type(
                    "hi.json",
                    "application/json",
                )),
                (_, _) => this.inner_handler.call(request).await,
            }
        })
    }
}
