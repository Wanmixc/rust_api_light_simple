FROM rust:1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust_api_light_simple /app/server
ENV HOST=0.0.0.0
ENV PORT=3000
ENV MAX_DB_CONNECTIONS=2
CMD ["/app/server"]
