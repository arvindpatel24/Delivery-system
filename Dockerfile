FROM rust:1.82-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY crates/ crates/
COPY migrations/ migrations/

RUN cargo build --release --bin delivery-api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/delivery-api /usr/local/bin/delivery-api

ENV RUST_LOG=info

EXPOSE 8080

CMD ["delivery-api"]
