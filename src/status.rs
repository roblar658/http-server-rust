use std::fmt;
use crate::error::HttpError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode {
    code: u16,
    reason: &'static str,
}

impl StatusCode {
    pub const OK: StatusCode = StatusCode { code: 200, reason: "OK" };
    pub const CREATED: StatusCode = StatusCode { code: 201, reason: "Created" };
    pub const ACCEPTED: StatusCode = StatusCode { code: 202, reason: "Accepted" };
    pub const NO_CONTENT: StatusCode = StatusCode { code: 204, reason: "No Content" };
    pub const MOVED_PERMANENTLY: StatusCode = StatusCode { code: 301, reason: "Moved Permanently" };
    pub const FOUND: StatusCode = StatusCode { code: 302, reason: "Found" };
    pub const NOT_MODIFIED: StatusCode = StatusCode { code: 304, reason: "Not Modified" };
    pub const BAD_REQUEST: StatusCode = StatusCode { code: 400, reason: "Bad Request" };
    pub const UNAUTHORIZED: StatusCode = StatusCode { code: 401, reason: "Unauthorized" };
    pub const FORBIDDEN: StatusCode = StatusCode { code: 403, reason: "Forbidden" };
    pub const NOT_FOUND: StatusCode = StatusCode { code: 404, reason: "Not Found" };
    pub const METHOD_NOT_ALLOWED: StatusCode = StatusCode { code: 405, reason: "Method Not Allowed" };
    pub const CONFLICT: StatusCode = StatusCode { code: 409, reason: "Conflict" };
    pub const INTERNAL_SERVER_ERROR: StatusCode = StatusCode { code: 500, reason: "Internal Server Error" };
    pub const NOT_IMPLEMENTED: StatusCode = StatusCode { code: 501, reason: "Not Implemented" };
    pub const BAD_GATEWAY: StatusCode = StatusCode { code: 502, reason: "Bad Gateway" };
    pub const SERVICE_UNAVAILABLE: StatusCode = StatusCode { code: 503, reason: "Service Unavailable" };

    pub const fn new(code: u16, reason: &'static str) -> Self {
        Self { code, reason }
    }

    pub fn as_u16(&self) -> u16 {
        self.code
    }

    pub fn reason_phrase(&self) -> &'static str {
        self.reason
    }

    pub fn from_u16(code: u16) -> Result<Self, HttpError> {
        match code {
            200 => Ok(Self::OK),
            201 => Ok(Self::CREATED),
            202 => Ok(Self::ACCEPTED),
            204 => Ok(Self::NO_CONTENT),
            301 => Ok(Self::MOVED_PERMANENTLY),
            302 => Ok(Self::FOUND),
            304 => Ok(Self::NOT_MODIFIED),
            400 => Ok(Self::BAD_REQUEST),
            401 => Ok(Self::UNAUTHORIZED),
            403 => Ok(Self::FORBIDDEN),
            404 => Ok(Self::NOT_FOUND),
            405 => Ok(Self::METHOD_NOT_ALLOWED),
            409 => Ok(Self::CONFLICT),
            500 => Ok(Self::INTERNAL_SERVER_ERROR),
            501 => Ok(Self::NOT_IMPLEMENTED),
            502 => Ok(Self::BAD_GATEWAY),
            503 => Ok(Self::SERVICE_UNAVAILABLE),
            c if (100..=999).contains(&c) => Ok(Self {
                code: c,
                reason: "Custom Status",
            }),
            _ => Err(HttpError::InvalidStatusCode(format!("Statuskode må være 100-999, fikk {code}"))),
        }
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code, self.reason)
    }
}
