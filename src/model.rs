use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidRequestLine,
    MissingHttpMethod,
    MissingRequestPath,
    UnrecognizedHttpMethod,
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidRequestLine => write!(f, "The request line was empty or invalid."),
            Error::MissingHttpMethod => {
                write!(f, "HTTP request line is missing the method (e.g., GET).")
            }
            Error::MissingRequestPath => {
                write!(f, "HTTP request line is missing the path (e.g., /).")
            }
            Error::UnrecognizedHttpMethod => write!(f, "HTTP method is not recognized."),
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl FromStr for HttpMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            _ => Err(Error::UnrecognizedHttpMethod),
        }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, path: &str) -> Self {
        Self {
            method,
            path: path.to_string(),
        }
    }
    pub fn method_and_path(&self) -> (HttpMethod, &str) {
        (self.method, &self.path)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HttpCode(u16);

impl HttpCode {
    pub fn ok() -> Self {
        Self(200)
    }

    pub fn not_found() -> Self {
        Self(404)
    }

    pub fn internal_server_error() -> Self {
        Self(500)
    }
}

impl Display for HttpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            200 => write!(f, "200 OK"),
            404 => write!(f, "404 NOT FOUND"),
            500 => write!(f, "500 INTERNAL SERVER ERROR"),
            _ => write!(f, "Undocumented HTTP code"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HttpStatus {
    Ok(HttpCode),
    AppError(HttpCode),
    ServerError(HttpCode),
}

#[derive(Debug)]
pub struct HttpResponse {
    status: HttpStatus,
    filename: String,
}

const HTTP_PROTOCOL_VERSION: &str = "HTTP/1.1";

impl Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            HttpStatus::Ok(code) => format!("{} {code}", HTTP_PROTOCOL_VERSION),
            HttpStatus::AppError(code) => format!("{} {code}", HTTP_PROTOCOL_VERSION),
            HttpStatus::ServerError(code) => format!("{} {code}", HTTP_PROTOCOL_VERSION),
        };
        write!(f, "{}", str)
    }
}

impl HttpResponse {
    pub fn new(status: HttpStatus, filename: &str) -> Self {
        Self {
            status,
            filename: filename.to_string(),
        }
    }

    pub fn ok(filename: &str) -> Self {
        Self::new(HttpStatus::Ok(HttpCode::ok()), filename)
    }

    pub fn not_found(filename: &str) -> Self {
        Self::new(HttpStatus::AppError(HttpCode::not_found()), filename)
    }

    pub fn internal_server_error(filename: &str) -> Self {
        Self::new(
            HttpStatus::ServerError(HttpCode::internal_server_error()),
            filename,
        )
    }

    pub fn status(&self) -> HttpStatus {
        self.status
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
}
