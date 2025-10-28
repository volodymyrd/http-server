use crate::model::{Error, Handler, HttpMethod, HttpRequest, HttpResponse};
use crate::server::Server;
use crate::{JsonContentType, RequestHandler};
use std::fmt::Debug;
use std::pin::Pin;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

async fn set_up<T>(handle_request: T) -> String
where
    T: Handler<HttpRequest, Response = HttpResponse> + Clone + Send + Sync + 'static,
    <T as Handler<HttpRequest>>::Future: Send,
    <T as Handler<HttpRequest>>::Error: Debug,
{
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let server = Server::new(listener);

    tokio::spawn(async move {
        server.run(handle_request).await.unwrap();
    });

    addr
}

#[tokio::test]
async fn test_server_responds_200_ok() {
    let handler = RequestHandler;
    let addr = set_up(handler).await;

    let response = send_request(&addr, "GET / HTTP/1.1\r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 200 OK");
}

#[tokio::test]
async fn test_server_responds_404_not_found() {
    let handler = RequestHandler;
    let addr = set_up(handler).await;

    let response = send_request(&addr, "GET /not_a_page HTTP/1.1\r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 404 NOT FOUND");
}

#[derive(Clone)]
struct RequestHandlerWithError;

impl Handler<HttpRequest> for RequestHandlerWithError {
    type Response = HttpResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = crate::model::Result<HttpResponse>> + Send>>;

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        Box::pin(async move { handle_request_with_error(request).await })
    }
}

async fn handle_request_with_error(_request: HttpRequest) -> crate::model::Result<HttpResponse> {
    Err(Error::App("Test error".to_string()))
}

#[tokio::test]
async fn test_server_responds_500_internal_server_error() {
    let handler = RequestHandlerWithError;
    let addr = set_up(handler).await;

    let response = send_request(&addr, "GET / \r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 500 INTERNAL SERVER ERROR");
}

#[derive(Clone)]
struct RequestHandlerWithTimeout;

impl Handler<HttpRequest> for RequestHandlerWithTimeout {
    type Response = HttpResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = crate::model::Result<HttpResponse>> + Send>>;
    fn call(&mut self, request: HttpRequest) -> Self::Future {
        Box::pin(async move { handle_request_with_timeout(request).await })
    }
}

async fn handle_request_with_timeout(request: HttpRequest) -> crate::model::Result<HttpResponse> {
    tokio::time::sleep(Duration::from_secs(3)).await;
    let response = match request.method_and_path() {
        (HttpMethod::Get, "/") => HttpResponse::ok("hello.html"),
        (_, _) => HttpResponse::not_found("404.html"),
    };

    Ok(response)
}

#[derive(Clone)]
struct Timeout<T> {
    // T will be some type that implements `Handler`
    inner_handler: T,
    duration: Duration,
}

impl<T> Timeout<T> {
    fn new(inner_handler: T, duration: Duration) -> Self {
        Self {
            inner_handler,
            duration,
        }
    }
}

impl<T> Handler<HttpRequest> for Timeout<T>
where
    T: Handler<HttpRequest, Response = HttpResponse, Error = Error> + Clone + Send + 'static,
    <T as Handler<HttpRequest>>::Future: Send,
{
    type Response = HttpResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = crate::model::Result<HttpResponse>> + Send>>;

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        // Get an owned clone of `&mut self`
        let mut this = self.clone();

        Box::pin(async move {
            let result =
                tokio::time::timeout(this.duration, this.inner_handler.call(request)).await;

            match result {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(error)) => Err(error),
                Err(_timeout) => Err(Error::App("Timeout exceeded".to_string())),
            }
        })
    }
}

#[tokio::test]
async fn test_server_responds_500_timeout() {
    let handler = RequestHandlerWithTimeout;
    let handler = Timeout::new(handler, Duration::from_secs(2));

    let addr = set_up(handler).await;

    let response = send_request(&addr, "GET / \r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 500 INTERNAL SERVER ERROR");
}

#[tokio::test]
async fn test_server_application_json_response() {
    let handler = RequestHandler;
    let handler = JsonContentType::new(handler);

    let addr = set_up(handler).await;

    let response = send_request(&addr, "GET /hi HTTP/1.1\r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 200 OK");
    assert_eq!(response[1].trim(), "Content-Length: 44");
    assert_eq!(response[2].trim(), "Content-Type: application/json");
}

/// A simple test client that connects, sends a request, and returns the first line of the response.
async fn send_request(addr: &str, request: &str) -> Vec<String> {
    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");

    // Send the HTTP request
    stream
        .write_all(request.as_bytes())
        .await
        .expect("Failed to write to stream");

    // Read the response
    let reader = BufReader::new(&mut stream);
    let mut lines = reader.lines();
    let mut response = Vec::new();
    while let Ok(Some(line)) = lines.next_line().await {
        response.push(line);
    }
    response
}
