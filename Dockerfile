# syntax=docker/dockerfile:1.7

FROM rust:1.95-slim AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates clang libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin kairos-server
COPY . .
RUN cargo build --release --bin kairos-server

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/kairos-server /usr/local/bin/kairos-server
USER nonroot
ENV PORT=8080 RUST_LOG=info,kairos=debug
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/kairos-server"]
