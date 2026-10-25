use std::fmt;
use std::io;

#[derive(Debug)]
pub enum HttpError {
    InvalidRequestLine(String),
    InvalidResponseLine(String),
    InvalidMethod(String),
    InvalidStatusCode(String),
    InvalidHeader(String),
    InvalidContentLength(String),
    IncompleteMessage,
    ConnectionClosed,
    Io(io::Error),
    UrlParseError(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::InvalidRequestLine(s) => write!(f, "Ugyldig request-linje: {s}"),
            HttpError::InvalidResponseLine(s) => write!(f, "Ugyldig response-linje: {s}"),
            HttpError::InvalidMethod(s) => write!(f, "Ugyldig HTTP-metode: {s}"),
            HttpError::InvalidStatusCode(s) => write!(f, "Ugyldig statuskode: {s}"),
            HttpError::InvalidHeader(s) => write!(f, "Ugyldig header: {s}"),
            HttpError::InvalidContentLength(s) => write!(f, "Ugyldig Content-Length: {s}"),
            HttpError::IncompleteMessage => write!(f, "Ufullstendig HTTP-melding"),
            HttpError::ConnectionClosed => write!(f, "Tilkobling lukket uventet"),
            HttpError::Io(e) => write!(f, "I/O-feil: {e}"),
            HttpError::UrlParseError(s) => write!(f, "URL parse-feil: {s}"),
        }
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HttpError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for HttpError {
    fn from(err: io::Error) -> Self {
        HttpError::Io(err)
    }
}
