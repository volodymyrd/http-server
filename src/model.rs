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

#[derive(Debug)]
pub struct HttpResponse {
    status_line: String,
    filename: String,
}

impl HttpResponse {
    pub fn new(status_line: &str, filename: &str) -> Self {
        Self {
            status_line: status_line.to_string(),
            filename: filename.to_string(),
        }
    }

    pub fn status_line(&self) -> &str {
        &self.status_line
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
}
