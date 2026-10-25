# Multi-stage Dockerfile for rust-http (Server og Klient)

# --- Byggesteg ---
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src
COPY tests ./tests

# Kjør tester under byggingen for å sikre protokollens korrekthet
RUN cargo test --release

# Bygg release-binærer for både server og klient
RUN cargo build --release --bin server --bin client

# --- Kjøresteg: Server ---
FROM alpine:latest AS server
WORKDIR /app

COPY --from=builder /app/target/release/server /usr/local/bin/server

EXPOSE 8080
ENV HOST=0.0.0.0
ENV PORT=8080

CMD ["server"]

# --- Kjøresteg: Klient ---
FROM alpine:latest AS client
WORKDIR /app

COPY --from=builder /app/target/release/client /usr/local/bin/client

ENV SERVER_URL=http://server:8080

CMD ["client"]
