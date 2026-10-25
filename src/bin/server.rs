use std::env;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Instant;
use rust_http::{HeaderMap, HttpError, Method, Request, Response, StatusCode, ThreadPool};

fn main() {
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("{host}:{port}");

    let listener = TcpListener::bind(&bind_addr).unwrap_or_else(|err| {
        eprintln!("[FEIL] Kunne ikke binde til {bind_addr}: {err}");
        std::process::exit(1);
    });

    let num_threads = 8;
    let pool = ThreadPool::new(num_threads);

    println!("=======================================================");
    println!("  🦀 Rust HTTP/1.1 Server starter");
    println!("  Lytter på: http://{bind_addr}");
    println!("  Trådpool:  {num_threads} arbeidere");
    println!("=======================================================");
    println!("Tilgjengelige ruter:");
    println!("  GET  /               - Dashboard i nettleser / API-oversikt");
    println!("  GET  /health         - Helsekontroll (200 OK JSON)");
    println!("  GET  /hello?name=X   - Tilpasset hilsen");
    println!("  POST /echo           - Speiler tilbake request-kropp");
    println!("  GET  /headers        - Viser mottatte HTTP-headere");
    println!("=======================================================");

    let start_time = Instant::now();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(move || {
                    handle_connection(stream, start_time);
                });
            }
            Err(e) => {
                eprintln!("[ADVARSEL] Feil ved mottak av tilkobling: {e}");
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, server_start: Instant) {
    let timeout = std::time::Duration::from_secs(10);
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));

    let peer_addr = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "ukjent".to_string());

    let mut reader = BufReader::new(&stream);
    let request_result = Request::parse(&mut reader);

    let (response, method_str, path_str) = match request_result {
        Ok(req) => {
            let m_str = req.method.to_string();
            let p_str = req.uri.clone();
            let res = route_request(&req, server_start);
            (res, m_str, p_str)
        }
        Err(HttpError::ConnectionClosed) => {
            return;
        }
        Err(e) => {
            eprintln!("[{peer_addr}] Parse-feil: {e}");
            let res = Response::bad_request(format!("Ugyldig HTTP-forespørsel: {e}"));
            (res, "BAD_REQ".to_string(), "-".to_string())
        }
    };

    let status_code = response.status.as_u16();
    let bytes = response.to_bytes();
    let body_len = response.body.len();

    if let Err(e) = stream.write_all(&bytes) {
        eprintln!("[{peer_addr}] Feil ved skriving av svar: {e}");
    }
    let _ = stream.flush();

    println!(
        "[{peer_addr}] \"{method_str} {path_str}\" -> {status_code} ({body_len} bytes)"
    );
}

fn route_request(req: &Request, server_start: Instant) -> Response {
    let path = req.path();

    match (&req.method, path) {
        (Method::Get, "/favicon.ico") => {
            Response::new(StatusCode::NO_CONTENT)
                .with_header(HeaderMap::CONNECTION, "close")
        }

        (Method::Get, "/") => {
            let accept = req.headers.get("Accept").unwrap_or("");
            if accept.contains("text/html") {
                let html = r#"<!DOCTYPE html>
<html lang="no">
<head>
  <meta charset="UTF-8">
  <title>🦀 Rust Custom HTTP/1.1 Server</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; background: #0f172a; color: #f8fafc; margin: 0; padding: 40px; }
    .card { background: #1e293b; border-radius: 12px; padding: 28px; max-width: 750px; margin: 0 auto; box-shadow: 0 10px 25px rgba(0,0,0,0.5); border: 1px solid #334155; }
    h1 { color: #f97316; margin-top: 0; display: flex; align-items: center; gap: 12px; }
    .badge { background: #059669; color: white; padding: 4px 10px; border-radius: 9999px; font-size: 0.8rem; font-weight: 600; }
    p { line-height: 1.6; color: #cbd5e1; }
    ul { list-style: none; padding: 0; }
    li { margin: 12px 0; background: #0f172a; padding: 12px 16px; border-radius: 8px; border: 1px solid #334155; display: flex; justify-content: space-between; align-items: center; }
    a { color: #38bdf8; text-decoration: none; font-weight: 500; }
    a:hover { text-decoration: underline; }
    .code { font-family: monospace; background: #020617; padding: 2px 6px; border-radius: 4px; color: #fdba74; }
    .form-group { margin-top: 24px; padding: 16px; background: #0f172a; border-radius: 8px; border: 1px solid #334155; }
    textarea { width: 100%; box-sizing: border-box; background: #1e293b; color: #fff; border: 1px solid #475569; border-radius: 6px; padding: 8px; font-family: monospace; }
    button { margin-top: 10px; background: #f97316; color: white; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; font-weight: 600; }
    button:hover { background: #ea580c; }
    pre { background: #020617; padding: 12px; border-radius: 6px; color: #a7f3d0; overflow-x: auto; display: none; margin-top: 12px; }
  </style>
</head>
<body>
  <div class="card">
    <h1>🦀 Rust HTTP/1.1 Server <span class="badge">Aktiv</span></h1>
    <p>Dette er en ren <strong>HTTP/1.1 protokollimplementasjon</strong> bygget fra bunnen av i <strong>Rust</strong> og kjører i <strong>Docker</strong> på Windows.</p>
    
    <h3>Testbare endepunkter:</h3>
    <ul>
      <li>
        <span><strong class="code">GET</strong> <a href="/health">/health</a></span>
        <small style="color: #94a3b8">Helsestatus og oppetid (JSON)</small>
      </li>
      <li>
        <span><strong class="code">GET</strong> <a href="/hello?name=Utvikler">/hello?name=Utvikler</a></span>
        <small style="color: #94a3b8">Query-parametre</small>
      </li>
      <li>
        <span><strong class="code">GET</strong> <a href="/headers">/headers</a></span>
        <small style="color: #94a3b8">Se nettleserens forespørselsheadere</small>
      </li>
      <li>
        <span><strong class="code">GET</strong> <a href="/">/?format=json</a></span>
        <small style="color: #94a3b8">Rå JSON API-oversikt</small>
      </li>
    </ul>

    <div class="form-group">
      <h4 style="margin-top: 0; color: #f97316;">Test POST /echo i sanntid:</h4>
      <textarea id="payload" rows="3">{"hilsen": "Hei fra nettleseren!", "klient": "browser-ui"}</textarea>
      <button onclick="sendEcho()">Send POST til /echo</button>
      <pre id="result"></pre>
    </div>
  </div>

  <script>
    async function sendEcho() {
      const payload = document.getElementById('payload').value;
      const resElem = document.getElementById('result');
      resElem.style.display = 'block';
      resElem.textContent = 'Sender forespørsel over HTTP/1.1...';
      try {
        const response = await fetch('/echo', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: payload
        });
        const data = await response.json();
        resElem.textContent = JSON.stringify(data, null, 2);
      } catch (err) {
        resElem.textContent = 'Feil: ' + err;
      }
    }
  </script>
</body>
</html>"#;
                Response::ok()
                    .html(html)
                    .with_header(HeaderMap::CONNECTION, "close")
            } else {
                let json = r#"{
  "service": "Rust Custom HTTP/1.1 Server",
  "status": "active",
  "routes": [
    {"method": "GET", "path": "/"},
    {"method": "GET", "path": "/health"},
    {"method": "GET", "path": "/hello?name=Verden"},
    {"method": "POST", "path": "/echo"},
    {"method": "GET", "path": "/headers"}
  ]
}"#;
                Response::ok()
                    .json(json)
                    .with_header(HeaderMap::CONNECTION, "close")
            }
        }

        (Method::Get, "/health") => {
            let uptime = server_start.elapsed().as_secs();
            let json = format!(
                r#"{{"status":"healthy","uptime_seconds":{uptime},"server":"rust-http-1.1"}}"#
            );
            Response::ok()
                .json(json)
                .with_header(HeaderMap::CONNECTION, "close")
        }

        (Method::Get, "/hello") => {
            let name = req.query_param("name").unwrap_or_else(|| "Verden".to_string());
            let json = format!(r#"{{"message":"Hei, {name}!","timestamp":"now"}}"#);
            Response::ok()
                .json(json)
                .with_header(HeaderMap::CONNECTION, "close")
        }

        (Method::Post, "/echo") => {
            let content_type = req
                .headers
                .get(HeaderMap::CONTENT_TYPE)
                .unwrap_or("application/octet-stream");
            let body_str = req.body_as_str().unwrap_or("[binær data]");

            let json = format!(
                r#"{{"echoed_body":{},"content_type":"{content_type}","length":{}}}"#,
                if body_str.starts_with('{') && body_str.ends_with('}') {
                    body_str.to_string()
                } else {
                    format!("\"{body_str}\"")
                },
                req.body.len()
            );

            Response::ok()
                .json(json)
                .with_header(HeaderMap::CONNECTION, "close")
        }

        (Method::Get, "/headers") => {
            let mut list = Vec::new();
            for (k, v) in req.headers.iter() {
                list.push(format!("    \"{k}\": \"{v}\""));
            }
            let json = format!("{{\n{}\n}}", list.join(",\n"));
            Response::ok()
                .json(json)
                .with_header(HeaderMap::CONNECTION, "close")
        }

        _ => {
            let json = format!(
                r#"{{"error":"404 Not Found","path":"{}","method":"{}"}}"#,
                req.uri, req.method
            );
            Response::not_found()
                .json(json)
                .with_header(HeaderMap::CONNECTION, "close")
        }
    }
}
