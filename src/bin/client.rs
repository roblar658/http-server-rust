use std::env;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use rust_http::{HttpClient, Method, Request};

fn main() {
    let args: Vec<String> = env::args().collect();
    let target_base = env::var("SERVER_URL")
        .unwrap_or_else(|_| {
            if args.len() > 1 && !args[1].starts_with('-') {
                args[1].clone()
            } else {
                "http://127.0.0.1:8080".to_string()
            }
        });

    println!("=======================================================");
    println!("  🚀 Rust Custom HTTP/1.1 Klient starter");
    println!("  Målserver: {target_base}");
    println!("=======================================================\n");

    let client = HttpClient::new();

    // Vent til serveren er oppe (nyttig i Docker Compose)
    wait_for_server(&client, &format!("{target_base}/health"), 15);

    println!("--- Test 1: GET /health ---");
    match client.get(&format!("{target_base}/health")) {
        Ok(res) => {
            println!("Status:   {}", res.status);
            println!("Headers:\n{}", res.headers);
            println!("Body:     {}", res.body_as_str().unwrap_or("[ugyldig utf8]"));
            assert_eq!(res.status.as_u16(), 200);
        }
        Err(e) => {
            eprintln!("Feilet: {e}");
            exit(1);
        }
    }
    println!("\n-------------------------------------------------------\n");

    println!("--- Test 2: GET / (Rot-oversikt) ---");
    match client.get(&format!("{target_base}/")) {
        Ok(res) => {
            println!("Status: {}", res.status);
            println!("Body:\n{}", res.body_as_str().unwrap_or(""));
            assert_eq!(res.status.as_u16(), 200);
        }
        Err(e) => {
            eprintln!("Feilet: {e}");
            exit(1);
        }
    }
    println!("\n-------------------------------------------------------\n");

    println!("--- Test 3: GET /hello?name=Rustentusiast (Query parameter) ---");
    match client.get(&format!("{target_base}/hello?name=Rustentusiast")) {
        Ok(res) => {
            println!("Status: {}", res.status);
            println!("Body:\n{}", res.body_as_str().unwrap_or(""));
            assert_eq!(res.status.as_u16(), 200);
        }
        Err(e) => {
            eprintln!("Feilet: {e}");
            exit(1);
        }
    }
    println!("\n-------------------------------------------------------\n");

    println!("--- Test 4: POST /echo (Sender JSON-kropp) ---");
    let payload = r#"{"melding": "Hallo fra containerisert Rust-klient!", "tall": 42}"#;
    match client.post(&format!("{target_base}/echo"), payload, "application/json") {
        Ok(res) => {
            println!("Status: {}", res.status);
            println!("Body:\n{}", res.body_as_str().unwrap_or(""));
            assert_eq!(res.status.as_u16(), 200);
        }
        Err(e) => {
            eprintln!("Feilet: {e}");
            exit(1);
        }
    }
    println!("\n-------------------------------------------------------\n");

    println!("--- Test 5: Egendefinert Request med egne headere ---");
    let (host, port, path) = HttpClient::parse_url(&format!("{target_base}/headers")).unwrap();
    let custom_req = Request::new(Method::Get, path)
        .with_header("X-Powered-By", "Rust-Custom-Protocol")
        .with_header("X-Client-Identifier", "client-worker-1");
    match client.send(custom_req, &host, port) {
        Ok(res) => {
            println!("Status: {}", res.status);
            println!("Body:\n{}", res.body_as_str().unwrap_or(""));
            assert_eq!(res.status.as_u16(), 200);
        }
        Err(e) => {
            eprintln!("Feilet: {e}");
            exit(1);
        }
    }
    println!("\n-------------------------------------------------------\n");

    println!("--- Test 6: GET /finnes-ikke (Forventer 404 Not Found) ---");
    match client.get(&format!("{target_base}/finnes-ikke")) {
        Ok(res) => {
            println!("Status: {}", res.status);
            println!("Body:\n{}", res.body_as_str().unwrap_or(""));
            assert_eq!(res.status.as_u16(), 404);
        }
        Err(e) => {
            eprintln!("Feilet: {e}");
            exit(1);
        }
    }
    println!("\n-------------------------------------------------------\n");

    println!("🎉 ALLE PROTOKOLLTESTER FULLFØRT UTEN FEIL!");
    println!("Klienten og serveren kommuniserte vellykket over tilpasset HTTP/1.1-protokoll!");
}

fn wait_for_server(client: &HttpClient, url: &str, max_retries: u32) {
    print!("Venter på at serveren skal bli klar...");
    for attempt in 1..=max_retries {
        match client.get(url) {
            Ok(res) if res.status.as_u16() == 200 => {
                println!(" [OK] (forsøk {attempt})");
                return;
            }
            _ => {
                sleep(Duration::from_millis(1000));
            }
        }
    }
    eprintln!("\n[ADVARSEL] Server svarte ikke innen tidsfristen, fortsetter likevel...");
}
