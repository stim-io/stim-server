# syntax=docker/dockerfile:1.7

FROM rust:1.90-bookworm AS builder

WORKDIR /app

ENV CARGO_HOME=/usr/local/cargo \
    CARGO_TARGET_DIR=/tmp/cargo-target

COPY modules/stim-server ./modules/stim-server
COPY modules/stim-proto ./modules/stim-proto

WORKDIR /app/modules/stim-server

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo fetch --locked

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/tmp/cargo-target \
    cargo build --release --bin server --locked \
    && mkdir -p /opt/artifacts \
    && cp /tmp/cargo-target/release/server /opt/artifacts/server

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends wget ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /opt/artifacts/server /usr/local/bin/stim-server

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080

CMD ["/usr/local/bin/stim-server"]
