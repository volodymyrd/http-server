use crate::model::{Error, HttpMethod, HttpRequest, HttpResponse};
use crate::server::Server;
use crate::{handle_request, handle_request_with_content_type};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

async fn set_up<F, Fut>(handle_request: F) -> String
where
    F: Fn(HttpRequest) -> Fut + Send + Sync + 'static + Clone,
    Fut: Future<Output = crate::model::Result<HttpResponse>> + Send,
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
    let addr = set_up(handle_request).await;

    let response = send_request(&addr, "GET / HTTP/1.1\r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 200 OK");
}

#[tokio::test]
async fn test_server_responds_404_not_found() {
    let addr = set_up(handle_request).await;

    let response = send_request(&addr, "GET /not_a_page HTTP/1.1\r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 404 NOT FOUND");
}

async fn handle_request_with_error(_request: HttpRequest) -> crate::model::Result<HttpResponse> {
    Err(Error::App("Test error".to_string()))
}

#[tokio::test]
async fn test_server_responds_500_internal_server_error() {
    let addr = set_up(handle_request_with_error).await;

    let response = send_request(&addr, "GET / \r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 500 INTERNAL SERVER ERROR");
}

async fn handle_request_with_timeout(request: HttpRequest) -> crate::model::Result<HttpResponse> {
    tokio::time::sleep(Duration::from_secs(3)).await;
    let response = match request.method_and_path() {
        (HttpMethod::Get, "/") => HttpResponse::ok("hello.html"),
        (_, _) => HttpResponse::not_found("404.html"),
    };

    Ok(response)
}

async fn handler_with_timeout(request: HttpRequest) -> crate::model::Result<HttpResponse> {
    let result =
        tokio::time::timeout(Duration::from_secs(2), handle_request_with_timeout(request)).await;

    match result {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(error)) => Err(error),
        Err(_timeout_elapsed) => Err(Error::App("Timeout exceeded".to_string())),
    }
}

#[tokio::test]
async fn test_server_responds_500_timeout() {
    let addr = set_up(handler_with_timeout).await;

    let response = send_request(&addr, "GET / \r\n").await;

    assert_eq!(response[0].trim(), "HTTP/1.1 500 INTERNAL SERVER ERROR");
}

#[tokio::test]
async fn test_server_application_json_response() {
    let addr = set_up(handle_request_with_content_type).await;

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
