FROM rust:slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY tool ./tool
COPY tool_derive ./tool_derive

RUN echo "fn main() {}" > dummy.rs
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
RUN rm dummy.rs

COPY src ./src

RUN cargo build --release --bin tadeusz

#####################################################################

FROM debian:trixie

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tadeusz /usr/local/bin/tadeusz

CMD ["tadeusz"]
