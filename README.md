# 🦀 Rust Custom HTTP/1.1 Protokoll (Server & Klient med Docker)

En ren, produksjonsklar og grundig implementasjon av **HTTP/1.1-protokollen (RFC 7230)** fra bunnen av i **Rust**, helt uten eksterne web-rammeverk som Actix, Hyper eller Reqwest. Prosjektet inneholder både en **flertrådet HTTP-server**, en **HTTP-klient** og en **Docker / Docker Compose**-arkitektur med helsesjekk.

Repository: [https://github.com/roblar658/http-server-rust](https://github.com/roblar658/http-server-rust)

---

## 🏛️ System- og Nettverksarkitektur

Følgende diagram viser hvordan klient, server, nettleser og Docker-nettverket samhandler:

```mermaid
flowchart TB
    subgraph Host["Vertsmaskin (Windows / Linux)"]
        Browser["Nettleser / curl<br/>http://localhost:8080/"]
    end

    subgraph DockerNet["Docker Nettverk: rust-http_default"]
        subgraph ServerContainer["Container: rust-http-server (Port 8080)"]
            Listener["TcpListener (0.0.0.0:8080)"]
            Pool["ThreadPool<br/>(8 Arbeidstråder + MPSC Kanal)"]
            
            subgraph HttpPipeline["HTTP/1.1 Protokollmotor"]
                Reader["BufReader (TCP Stream)"]
                Parser["Request Parser<br/>(Metode, URL, Headere, Body)"]
                Router{"Ruter / Dispatcher"}
                Serializer["Response Serializer<br/>(Statuslinje, Headere, Body)"]
            end
            
            subgraph Routes["Endepunkter"]
                R1["GET / (HTML Dashboard / JSON)"]
                R2["GET /health (Helsesjekk)"]
                R3["GET /hello?name=X (Query Param)"]
                R4["POST /echo (JSON / Tekst Body)"]
                R5["GET /headers (Ekko av klientheadere)"]
                R6["* (404 Not Found)"]
            end
        end

        subgraph ClientContainer["Container: rust-http-client"]
            ClientRunner["Test Runner CLI"]
            HttpClientModule["HttpClient Modul<br/>(URL-parser, TCP Socket, Request Builder)"]
        end
    end

    Browser -->|Port 8080:8080| Listener
    ClientContainer -->|http://server:8080| Listener
    Listener -->|Aksepterer TCP Stream| Pool
    Pool -->|Tildeler tråd| Reader
    Reader --> Parser
    Parser --> Router
    Router --> R1
    Router --> R2
    Router --> R3
    Router --> R4
    Router --> R5
    Router --> R6
    Routes --> Serializer
    Serializer -->|Rå HTTP Bytes over TCP| Listener
```

---

## 🔄 Protokoll-Livssyklus (Sekvensdiagram)

Dette diagrammet illustrerer overføringen av data over rå TCP-sockets når en HTTP-forespørsel utføres og besvares:

```mermaid
sequenceDiagram
    autonumber
    participant Client as HTTP-Klient / Nettleser
    participant Socket as TCP Socket (Port 8080)
    participant Worker as Server Arbeidstråd
    participant Engine as Protokollmotor og Ruter

    Client->>Socket: TCP 3-veis håndtrykk (SYN, SYN-ACK, ACK)
    Client->>Socket: Sender HTTP Request (POST /echo)
    Note over Client,Socket: POST /echo HTTP/1.1<br/>Host: server:8080<br/>Content-Type: application/json<br/>Content-Length: 32<br/>Connection: close<br/><br/>{"melding":"Hei fra klienten!"}

    Socket->>Worker: Tildeler aktiv TcpStream fra ThreadPool
    Worker->>Engine: Request::parse(&mut BufReader)
    Note over Engine: Leser requestlinje, parser headere til HeaderMap,<br/>leser nøyaktig Content-Length bytes med body
    Engine->>Engine: Ruter matcher POST /echo
    Engine->>Worker: Genererer Response::ok().json(...)

    Worker->>Socket: Sender HTTP Response (200 OK)
    Note over Socket,Worker: HTTP/1.1 200 OK<br/>Server: RustCustomHttp/1.0<br/>Content-Type: application/json<br/>Content-Length: 78<br/>Connection: close<br/><br/>{"echoed_body":{...},"length":32}

    Socket->>Client: Mottar HTTP-svar over nettverket
    Client->>Client: Response::parse(&mut BufReader)
    Socket-->>Client: Lukker TCP-tilkobling (Connection: close)
```

---

## 🧩 Datamodell & Modulstruktur

```mermaid
classDiagram
    class Request {
        +Method method
        +String uri
        +String version
        +HeaderMap headers
        +Vec body
        +new(method, uri) Request
        +path() str
        +query_param(key) Option
        +body_as_str() Option
        +to_bytes() Vec
        +parse(reader) Result
    }

    class Response {
        +String version
        +StatusCode status
        +HeaderMap headers
        +Vec body
        +ok() Response
        +not_found() Response
        +bad_request(msg) Response
        +json(json_str) Response
        +html(html_str) Response
        +to_bytes() Vec
        +parse(reader) Result
    }

    class HeaderMap {
        -BTreeMap headers
        +insert(key, value)
        +get(key) Option
        +content_length() Option
        +iter() Iterator
    }

    class Method {
        <<enumeration>>
        Get
        Post
        Put
        Delete
        Head
        Options
        Patch
        Custom
        +as_str() str
    }

    class StatusCode {
        +u16 code
        +str reason
        +as_u16() u16
        +reason_phrase() str
        +from_u16(code) Result
    }

    class HttpClient {
        +u64 default_timeout_ms
        +parse_url(url) Result
        +send(req, host, port) Result
        +get(url) Result
        +post(url, body, content_type) Result
    }

    class ThreadPool {
        -Vec workers
        -Sender sender
        +new(size) ThreadPool
        +execute(job)
    }

    Request --> Method
    Request --> HeaderMap
    Response --> StatusCode
    Response --> HeaderMap
    HttpClient ..> Request
    HttpClient ..> Response
```

---

## 📦 Docker Bygge- og Kjøringspipeline

```mermaid
flowchart LR
    subgraph BuildStage["Byggesteg (rust:alpine)"]
        SRC["Rust Kildekode<br/>(src/, tests/, Cargo.toml)"]
        TESTS["cargo test --release<br/>(Protokoll- og integrasjonstester)"]
        BUILD["cargo build --release<br/>(Kompilerer server og klient)"]
        SRC --> TESTS --> BUILD
    end

    subgraph RuntimeServer["Kjørebilde 1: rust-http-server (alpine)"]
        SRV_BIN["Kopiert binærfil:<br/>/usr/local/bin/server"]
        SRV_PORT["EXPOSE 8080<br/>CMD ['server']"]
        SRV_BIN --> SRV_PORT
    end

    subgraph RuntimeClient["Kjørebilde 2: rust-http-client (alpine)"]
        CLI_BIN["Kopiert binærfil:<br/>/usr/local/bin/client"]
        CLI_RUN["CMD ['client']<br/>Kobler til http://server:8080"]
        CLI_BIN --> CLI_RUN
    end

    BUILD -->|Kopierer server binær| SRV_BIN
    BUILD -->|Kopierer klient binær| CLI_BIN
```

---

## 📂 Fil- og Mappestruktur

```text
C:\dev\rust-http\
├── Cargo.toml                  # Prosjektkonfigurasjon (server + client binærer)
├── Dockerfile                  # Multi-stage Dockerfile med testkjøring og minimale Alpine-bilder
├── docker-compose.yml          # Orkestrering med helsesjekk og automatisk klienttest
├── run.ps1                     # PowerShell-snarvei for bygging og oppstart
├── README.md                   # Arkitektur- og brukerveiledning
├── src/
│   ├── lib.rs                  # Felles kjernebibliotek og enhetstester
│   ├── error.rs                # Egendefinert HttpError feilhåndtering
│   ├── method.rs               # HTTP-metoder (GET, POST, PUT, DELETE, etc.)
│   ├── status.rs               # HTTP statuskoder (200 OK, 404, 500, etc.)
│   ├── headers.rs              # Case-insensitiv HeaderMap (RFC 7230)
│   ├── request.rs              # Request-parser med query-parametre og body
│   ├── response.rs             # Response-parser og byte-serialisering
│   ├── threadpool.rs           # Trådpool for flertrådet TCP-server
│   ├── client.rs               # HttpClient for sending av HTTP-forespørsler
│   └── bin/
│       ├── server.rs           # HTTP-server som lytter på 0.0.0.0:8080 med HTML UI og ruter
│       └── client.rs           # HTTP-klient CLI med automatisert testsuite
└── tests/
    └── protocol_tests.rs       # Integrasjonstester (norske tegn, URL-parsing, m.m.)
```

---

## 🚀 Kjøring

### 1. Start med Docker (Anbefalt)

```powershell
cd C:\dev\rust-http
.\run.ps1
```

Eller direkte via Docker:

```powershell
# Start serveren i bakgrunnen
docker compose up -d server

# Kjør klienttesten
docker compose run --rm client

# Stopp serveren når ferdig
docker compose down
```

---

## 🌐 Test i nettleser eller med curl

Når serveren kjører i Docker, åpne:
- **Dashboard & Tester:** [http://localhost:8080/](http://localhost:8080/)
- **Helsestatus:** [http://localhost:8080/health](http://localhost:8080/health)
- **Query-parameter:** [http://localhost:8080/hello?name=Verden](http://localhost:8080/hello?name=Verden)
- **Ekko med curl:**
  ```powershell
  curl -X POST http://localhost:8080/echo -H "Content-Type: application/json" -d '{"melding":"hei!"}'
  ```

---

## 💻 Kjøring lokalt med Cargo

Dersom Rust er installert lokalt på maskinen:

```powershell
# Kjøre alle tester
cargo test

# Starte serveren
cargo run --bin server

# Kjøre klienten (i et annet terminalvindu)
cargo run --bin client
```
