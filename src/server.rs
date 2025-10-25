use crate::model::{Error, HttpRequest, HttpResponse, Result};
use crate::utils::extract_http_details;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[derive(Debug)]
pub(crate) struct Server {
    listener: TcpListener,
}

impl Server {
    pub(crate) fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub(crate) async fn run<F, Fut>(&self, handler: F) -> Result<()>
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<HttpResponse>> + Send,
    {
        loop {
            let (mut stream, _) = self.listener.accept().await.map_err(Error::Io)?;

            let request = Self::read_http_request(&mut stream).await?;

            let handler = handler.clone();

            tokio::spawn(async move {
                let response = handler(request).await.unwrap_or_else(|e| {
                    eprintln!("Error handling request: {:?}", e);
                    HttpResponse::internal_server_error("500.html")
                });

                match Self::write_http_response(&mut stream, response).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error writing HTTP response: {:?}", e);
                    }
                }
            });
        }
    }

    async fn read_http_request(
        mut stream: impl AsyncRead + AsyncWrite + Unpin,
    ) -> Result<HttpRequest> {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        buf_reader
            .read_line(&mut request_line)
            .await
            .map_err(Error::Io)?;
        let (method, path) = extract_http_details(&request_line)?;
        let request = HttpRequest::new(method, path);
        Ok(request)
    }

    async fn write_http_response(
        mut stream: impl AsyncWrite + Unpin,
        response: HttpResponse,
    ) -> std::io::Result<()> {
        let contents = fs::read_to_string(response.filename()).await?;
        let length = contents.len();

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            response.status(),
            length,
            contents
        );

        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::HttpMethod;
    use std::io::Cursor;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, ReadBuf};

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
    async fn test_read_http_request() {
        let mut stream = MockStream::new("GET /test HTTP/1.1\r\n");
        let request = Server::read_http_request(&mut stream).await.unwrap();
        assert_eq!(request.method_and_path(), (HttpMethod::Get, "/test"));
    }

    #[tokio::test]
    async fn test_write_http_response() {
        let file_name = "test1.html";
        let mut stream = MockStream::new("");
        let response = HttpResponse::ok(file_name);

        fs::write(file_name, "Test content").await.unwrap();

        Server::write_http_response(&mut stream, response)
            .await
            .unwrap();

        let response_str = String::from_utf8(stream.writer).unwrap();
        assert_eq!(
            response_str,
            "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nTest content"
        );

        fs::remove_file(file_name).await.unwrap();
    }

    #[tokio::test]
    async fn test_write_http_response_internal_server_error() {
        let file_name = "test2.html";
        let mut stream = MockStream::new("");
        let response = HttpResponse::internal_server_error(file_name);

        fs::write(file_name, "Internal Server Error").await.unwrap();

        Server::write_http_response(&mut stream, response)
            .await
            .unwrap();

        let response_str = String::from_utf8(stream.writer).unwrap();
        assert_eq!(
            response_str,
            "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 21\r\n\r\nInternal Server Error"
        );

        fs::remove_file(file_name).await.unwrap();
    }
}
