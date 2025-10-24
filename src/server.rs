use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

pub(crate) struct Server {
    listener: TcpListener,
}

impl Server {
    pub(crate) fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub(crate) fn run(&self) -> Result<()> {
        for stream in self.listener.incoming() {
            let stream = stream.map_err(Error::Io)?;
            self.handle_connection(stream);
        }
        Ok(())
    }

    fn handle_connection<T: Read + Write>(&self, mut stream: T) {
        let buf_reader = BufReader::new(&mut stream);
        let request_line = buf_reader.lines().next().unwrap().unwrap();

        let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
            ("HTTP/1.1 200 OK", "hello.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "404.html")
        };

        let contents = fs::read_to_string(filename).unwrap();
        let length = contents.len();

        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes()).unwrap();
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // A mock stream that uses in-memory vectors for reading and writing.
    // This allows us to simulate a network connection without any actual I/O.
    struct MockStream {
        read_data: Vec<u8>,
        write_data: Vec<u8>,
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let size = std::cmp::min(buf.len(), self.read_data.len());
            buf[..size].copy_from_slice(&self.read_data[..size]);
            self.read_data = self.read_data.split_off(size);
            Ok(size)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_handle_connection_200_unit() {
        // Simulate a client sending a valid request
        let request = b"GET / HTTP/1.1\r\n";
        let mut stream = MockStream {
            read_data: request.to_vec(),
            write_data: Vec::new(),
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let server = Server::new(listener);

        server.handle_connection(&mut stream);

        let contents = fs::read_to_string("hello.html").unwrap();
        let expected_response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );

        let response = String::from_utf8(stream.write_data).unwrap();
        assert_eq!(response, expected_response);
    }

    #[test]
    fn test_handle_connection_404_unit() {
        let request = b"GET /invalid_path HTTP/1.1\r\n";
        let mut stream = MockStream {
            read_data: request.to_vec(),
            write_data: Vec::new(),
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let server = Server::new(listener);

        server.handle_connection(&mut stream);

        let contents = fs::read_to_string("404.html").unwrap();
        let expected_response = format!(
            "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );

        let response = String::from_utf8(stream.write_data).unwrap();
        assert_eq!(response, expected_response);
    }
}
