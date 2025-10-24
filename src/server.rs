use std::fmt::{Display, Formatter};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

pub(crate) struct Server {
    listener: TcpListener,
}

impl Server {
    pub(crate) fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub(crate) async fn run(&self) -> Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await.map_err(Error::Io)?;
            tokio::spawn(async move {
                if let Err(e) = Server::handle_connection(stream).await {
                    eprintln!("failed to handle connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut stream: impl AsyncRead + AsyncWrite + Unpin,
    ) -> std::io::Result<()> {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        buf_reader.read_line(&mut request_line).await?;

        let (status_line, filename) = if request_line == "GET / HTTP/1.1\r\n" {
            ("HTTP/1.1 200 OK", "hello.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "404.html")
        };

        let contents = fs::read_to_string(filename).await?;
        let length = contents.len();

        let response =
            format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
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
    use std::io::Cursor;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    struct MockStream {
        reader: Cursor<Vec<u8>>,
        writer: Vec<u8>,
    }

    impl Unpin for MockStream {}

    impl MockStream {
        fn new(request: &str) -> Self {
            MockStream {
                reader: Cursor::new(request.as_bytes().to_vec()),
                writer: Vec::new(),
            }
        }
    }

    impl AsyncRead for MockStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let this = self.get_mut();
            let pos = this.reader.position() as usize;
            let remaining_data = &this.reader.get_ref()[pos..];
            let len = std::cmp::min(remaining_data.len(), buf.remaining());
            buf.put_slice(&remaining_data[..len]);
            this.reader.set_position((pos + len) as u64);
            Poll::Ready(Ok(()))
        }
    }

    impl AsyncWrite for MockStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.get_mut().writer.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn test_handle_connection_get_request() {
        let mut stream = MockStream::new("GET / HTTP/1.1\r\n");

        fs::write("hello.html", "<html><body>Hello</body></html>")
            .await
            .unwrap();

        Server::handle_connection(&mut stream).await.unwrap();

        let response = String::from_utf8(stream.writer).unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("<html><body>Hello</body></html>"));
    }

    #[tokio::test]
    async fn test_handle_connection_not_found_request() {
        let mut stream = MockStream::new("GET /notfound HTTP/1.1\r\n");

        fs::write("404.html", "<html><body>404 Not Found</body></html>")
            .await
            .unwrap();

        Server::handle_connection(&mut stream).await.unwrap();

        let response = String::from_utf8(stream.writer).unwrap();
        assert!(response.starts_with("HTTP/1.1 404 NOT FOUND"));
        assert!(response.contains("<html><body>404 Not Found</body></html>"));
    }
}
