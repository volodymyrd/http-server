use crate::server::Server;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

fn set_up() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let server = Server::new(listener);

    let _ = thread::spawn(move || {
        server.run().unwrap();
    });

    addr
}

#[test]
fn test_server_responds_200_ok() {
    let addr = set_up();

    let response = send_request(&addr, "GET / HTTP/1.1\r\n");

    assert_eq!(response.trim(), "HTTP/1.1 200 OK");
}

#[test]
fn test_server_responds_404_not_found() {
    let addr = set_up();

    let response = send_request(&addr, "GET /not_a_page HTTP/1.1\r\n");

    assert_eq!(response.trim(), "HTTP/1.1 404 NOT FOUND");
}

/// A simple test client that connects, sends a request, and returns the first line of the response.
fn send_request(addr: &str, request: &str) -> String {
    let mut stream = TcpStream::connect(addr).expect("Failed to connect to server");

    // Send the HTTP request
    stream
        .write_all(request.as_bytes())
        .expect("Failed to write to stream");

    // Read the response
    let mut reader = BufReader::new(&stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read from stream");

    response_line
}
