use std::io::BufRead;
use crate::error::HttpError;
use crate::headers::HeaderMap;
use crate::method::Method;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: Method,
    pub uri: String,
    pub version: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Request {
    pub fn new(method: Method, uri: impl Into<String>) -> Self {
        let uri_str = uri.into();
        Self {
            method,
            uri: uri_str,
            version: "HTTP/1.1".to_string(),
            headers: HeaderMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self.headers.insert(HeaderMap::CONTENT_LENGTH, self.body.len().to_string());
        self
    }

    pub fn body_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }

    pub fn path(&self) -> &str {
        if let Some(pos) = self.uri.find('?') {
            &self.uri[..pos]
        } else {
            &self.uri
        }
    }

    pub fn query_param(&self, key: &str) -> Option<String> {
        let query_str = self.uri.find('?').map(|pos| &self.uri[pos + 1..])?;
        for pair in query_str.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(k) = parts.next() {
                if k == key {
                    return parts.next().map(|v| v.to_string()).or_else(|| Some(String::new()));
                }
            }
        }
        None
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut headers = self.headers.clone();
        if !headers.contains_key(HeaderMap::CONTENT_LENGTH) && !self.body.is_empty() {
            headers.insert(HeaderMap::CONTENT_LENGTH, self.body.len().to_string());
        }

        let mut out = Vec::new();
        let request_line = format!("{} {} {}\r\n", self.method, self.uri, self.version);
        out.extend_from_slice(request_line.as_bytes());

        for (k, v) in headers.iter() {
            let h = format!("{k}: {v}\r\n");
            out.extend_from_slice(h.as_bytes());
        }

        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(&self.body);
        out
    }

    pub fn parse<R: BufRead>(reader: &mut R) -> Result<Self, HttpError> {
        let mut request_line = String::new();
        let bytes_read = reader.read_line(&mut request_line)?;
        if bytes_read == 0 {
            return Err(HttpError::ConnectionClosed);
        }

        let trimmed_line = request_line.trim_end_matches(&['\r', '\n'][..]);
        if trimmed_line.is_empty() {
            return Err(HttpError::IncompleteMessage);
        }

        let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(HttpError::InvalidRequestLine(trimmed_line.to_string()));
        }

        let method: Method = parts[0].parse()?;
        let uri = parts[1].to_string();
        let version = parts[2].to_string();

        let mut headers = HeaderMap::new();
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                break;
            }
            let line_trimmed = line.trim_end_matches(&['\r', '\n'][..]);
            if line_trimmed.is_empty() {
                break;
            }

            if let Some((name, val)) = line_trimmed.split_once(':') {
                headers.insert(name.trim(), val.trim());
            } else {
                return Err(HttpError::InvalidHeader(line_trimmed.to_string()));
            }
        }

        let content_length = headers.content_length().unwrap_or(0);
        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            reader.read_exact(&mut body)?;
        }

        Ok(Request {
            method,
            uri,
            version,
            headers,
            body,
        })
    }
}
