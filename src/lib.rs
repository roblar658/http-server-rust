pub mod client;
pub mod error;
pub mod headers;
pub mod method;
pub mod request;
pub mod response;
pub mod status;
pub mod threadpool;

pub use client::HttpClient;
pub use error::HttpError;
pub use headers::HeaderMap;
pub use method::Method;
pub use request::Request;
pub use response::Response;
pub use status::StatusCode;
pub use threadpool::ThreadPool;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_get_request() {
        let raw = b"GET /index.html?user=alice HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: TestAgent\r\n\r\n";
        let mut cursor = Cursor::new(raw);
        let req = Request::parse(&mut cursor).expect("Failed to parse GET request");

        assert_eq!(req.method, Method::Get);
        assert_eq!(req.uri, "/index.html?user=alice");
        assert_eq!(req.path(), "/index.html");
        assert_eq!(req.query_param("user"), Some("alice".to_string()));
        assert_eq!(req.headers.get("Host"), Some("localhost:8080"));
        assert_eq!(req.headers.get("user-agent"), Some("TestAgent"));
        assert!(req.body.is_empty());
    }

    #[test]
    fn test_parse_post_request_with_body() {
        let payload = r#"{"name":"Rust HTTP"}"#;
        let raw = format!(
            "POST /api/echo HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            payload.len(),
            payload
        );
        let mut cursor = Cursor::new(raw.into_bytes());
        let req = Request::parse(&mut cursor).expect("Failed to parse POST request");

        assert_eq!(req.method, Method::Post);
        assert_eq!(req.path(), "/api/echo");
        assert_eq!(req.body_as_str(), Some(payload));
    }

    #[test]
    fn test_parse_response() {
        let body = "Hei verden!";
        let raw = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let mut cursor = Cursor::new(raw.into_bytes());
        let res = Response::parse(&mut cursor).expect("Failed to parse response");

        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(res.headers.get("content-type"), Some("text/plain; charset=utf-8"));
        assert_eq!(res.body_as_str(), Some(body));
    }

    #[test]
    fn test_request_roundtrip() {
        let original = Request::new(Method::Post, "/submit")
            .with_header("X-Custom-Header", "CustomValue")
            .with_body(b"Roundtrip data".to_vec());

        let bytes = original.to_bytes();
        let mut cursor = Cursor::new(bytes);
        let parsed = Request::parse(&mut cursor).expect("Failed roundtrip parse");

        assert_eq!(parsed.method, original.method);
        assert_eq!(parsed.uri, original.uri);
        assert_eq!(parsed.headers.get("x-custom-header"), Some("CustomValue"));
        assert_eq!(parsed.body, original.body);
    }
}
