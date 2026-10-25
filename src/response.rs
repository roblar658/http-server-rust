use std::io::BufRead;
use crate::error::HttpError;
use crate::headers::HeaderMap;
use crate::status::StatusCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub version: String,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(HeaderMap::SERVER, "RustCustomHttp/1.0");
        Self {
            version: "HTTP/1.1".to_string(),
            status,
            headers,
            body: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    pub fn created() -> Self {
        Self::new(StatusCode::CREATED)
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND).text("404 Not Found")
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST).text(msg)
    }

    pub fn internal_server_error(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR).text(msg)
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

    pub fn text(self, text: impl Into<String>) -> Self {
        let s = text.into();
        self.with_header(HeaderMap::CONTENT_TYPE, "text/plain; charset=utf-8")
            .with_body(s.into_bytes())
    }

    pub fn json(self, json_str: impl Into<String>) -> Self {
        let s = json_str.into();
        self.with_header(HeaderMap::CONTENT_TYPE, "application/json; charset=utf-8")
            .with_body(s.into_bytes())
    }

    pub fn html(self, html_str: impl Into<String>) -> Self {
        let s = html_str.into();
        self.with_header(HeaderMap::CONTENT_TYPE, "text/html; charset=utf-8")
            .with_body(s.into_bytes())
    }

    pub fn body_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut headers = self.headers.clone();
        if !headers.contains_key(HeaderMap::CONTENT_LENGTH) {
            headers.insert(HeaderMap::CONTENT_LENGTH, self.body.len().to_string());
        }

        let mut out = Vec::new();
        let status_line = format!(
            "{} {} {}\r\n",
            self.version,
            self.status.as_u16(),
            self.status.reason_phrase()
        );
        out.extend_from_slice(status_line.as_bytes());

        for (k, v) in headers.iter() {
            let h = format!("{k}: {v}\r\n");
            out.extend_from_slice(h.as_bytes());
        }

        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(&self.body);
        out
    }

    pub fn parse<R: BufRead>(reader: &mut R) -> Result<Self, HttpError> {
        let mut status_line = String::new();
        let bytes_read = reader.read_line(&mut status_line)?;
        if bytes_read == 0 {
            return Err(HttpError::ConnectionClosed);
        }

        let trimmed_line = status_line.trim_end_matches(&['\r', '\n'][..]);
        if trimmed_line.is_empty() {
            return Err(HttpError::IncompleteMessage);
        }

        let parts: Vec<&str> = trimmed_line.splitn(3, ' ').collect();
        if parts.len() < 2 {
            return Err(HttpError::InvalidResponseLine(trimmed_line.to_string()));
        }

        let version = parts[0].to_string();
        let status_code_num: u16 = parts[1]
            .parse()
            .map_err(|_| HttpError::InvalidStatusCode(parts[1].to_string()))?;
        let status = StatusCode::from_u16(status_code_num)?;

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

        let mut body = Vec::new();
        if let Some(content_length) = headers.content_length() {
            body.resize(content_length, 0);
            if content_length > 0 {
                reader.read_exact(&mut body)?;
            }
        } else if status.as_u16() != 204 && status.as_u16() != 304 {
            reader.read_to_end(&mut body)?;
        }

        Ok(Response {
            version,
            status,
            headers,
            body,
        })
    }
}
