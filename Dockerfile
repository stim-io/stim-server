FROM rust:1.90-bookworm AS builder
WORKDIR /app

COPY modules/stim-server ./modules/stim-server
COPY modules/stim-proto ./modules/stim-proto
WORKDIR /app/modules/stim-server
RUN cargo build --release --bin server

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends wget ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/modules/stim-server/target/release/server /usr/local/bin/stim-server

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080

CMD ["/usr/local/bin/stim-server"]
