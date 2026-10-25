use std::io::{BufReader, Write};
use std::net::TcpStream;
use crate::error::HttpError;
use crate::headers::HeaderMap;
use crate::method::Method;
use crate::request::Request;
use crate::response::Response;

pub struct HttpClient {
    pub default_timeout_ms: u64,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            default_timeout_ms: 10_000,
        }
    }

    pub fn parse_url(url: &str) -> Result<(String, u16, String), HttpError> {
        let trimmed = url.trim();
        let without_proto = if let Some(stripped) = trimmed.strip_prefix("http://") {
            stripped
        } else if trimmed.starts_with("https://") {
            return Err(HttpError::UrlParseError("HTTPS krever TLS/SSL; bruk HTTP i denne implementasjonen".into()));
        } else {
            trimmed
        };

        let (host_port, path) = match without_proto.find('/') {
            Some(idx) => (&without_proto[..idx], &without_proto[idx..]),
            None => (without_proto, "/"),
        };

        let (host, port) = match host_port.find(':') {
            Some(idx) => {
                let h = &host_port[..idx];
                let p = host_port[idx + 1..]
                    .parse::<u16>()
                    .map_err(|_| HttpError::UrlParseError(format!("Ugyldig portnummer i {host_port}")))?;
                (h, p)
            }
            None => (host_port, 80),
        };

        if host.is_empty() {
            return Err(HttpError::UrlParseError("Vertsnavn mangler i URL".into()));
        }

        Ok((host.to_string(), port, path.to_string()))
    }

    pub fn send(&self, mut req: Request, host: &str, port: u16) -> Result<Response, HttpError> {
        if !req.headers.contains_key(HeaderMap::HOST) {
            req.headers.insert(HeaderMap::HOST, format!("{host}:{port}"));
        }
        if !req.headers.contains_key(HeaderMap::USER_AGENT) {
            req.headers.insert(HeaderMap::USER_AGENT, "RustCustomHttpClient/1.0");
        }
        if !req.headers.contains_key(HeaderMap::CONNECTION) {
            req.headers.insert(HeaderMap::CONNECTION, "close");
        }

        let address = format!("{host}:{port}");
        let mut stream = TcpStream::connect(&address)?;
        
        let timeout = std::time::Duration::from_millis(self.default_timeout_ms);
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));

        let raw_bytes = req.to_bytes();
        stream.write_all(&raw_bytes)?;
        stream.flush()?;

        let mut reader = BufReader::new(stream);
        let response = Response::parse(&mut reader)?;
        Ok(response)
    }

    pub fn get(&self, url: &str) -> Result<Response, HttpError> {
        let (host, port, path) = Self::parse_url(url)?;
        let req = Request::new(Method::Get, path);
        self.send(req, &host, port)
    }

    pub fn post(&self, url: &str, body: impl Into<Vec<u8>>, content_type: &str) -> Result<Response, HttpError> {
        let (host, port, path) = Self::parse_url(url)?;
        let req = Request::new(Method::Post, path)
            .with_header(HeaderMap::CONTENT_TYPE, content_type)
            .with_body(body);
        self.send(req, &host, port)
    }
}
