use std::io::Cursor;
use rust_http::{HeaderMap, HttpClient, Method, Request, StatusCode};

#[test]
fn test_case_insensitive_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/plain");
    headers.insert("X-API-KEY", "secret123");

    assert_eq!(headers.get("content-type"), Some("text/plain"));
    assert_eq!(headers.get("CONTENT-TYPE"), Some("text/plain"));
    assert_eq!(headers.get("Content-Type"), Some("text/plain"));
    assert_eq!(headers.get("x-api-key"), Some("secret123"));
}

#[test]
fn test_norwegian_characters_in_payload() {
    let text = "Blåbærsyltetøy og rømmegrøt på en søndag";
    let req = Request::new(Method::Post, "/norsk")
        .with_header("Content-Type", "text/plain; charset=utf-8")
        .with_body(text.as_bytes().to_vec());

    let raw = req.to_bytes();
    let mut cursor = Cursor::new(raw);
    let parsed = Request::parse(&mut cursor).expect("Kunne ikke parse norsk payload");

    assert_eq!(parsed.body_as_str(), Some(text));
    assert_eq!(parsed.headers.content_length(), Some(text.len()));
}

#[test]
fn test_parse_url() {
    let (host, port, path) = HttpClient::parse_url("http://api.eksempel.no:9000/v1/brukere?id=10").unwrap();
    assert_eq!(host, "api.eksempel.no");
    assert_eq!(port, 9000);
    assert_eq!(path, "/v1/brukere?id=10");

    let (host2, port2, path2) = HttpClient::parse_url("http://localhost/health").unwrap();
    assert_eq!(host2, "localhost");
    assert_eq!(port2, 80);
    assert_eq!(path2, "/health");
}

#[test]
fn test_status_codes() {
    assert_eq!(StatusCode::OK.as_u16(), 200);
    assert_eq!(StatusCode::OK.reason_phrase(), "OK");
    assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);
    assert_eq!(StatusCode::NOT_FOUND.reason_phrase(), "Not Found");
    assert_eq!(StatusCode::from_u16(500).unwrap(), StatusCode::INTERNAL_SERVER_ERROR);
}
