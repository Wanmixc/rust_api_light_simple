# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /app

# Cache dependency builds separately from app source.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
COPY tests ./tests
RUN touch src/main.rs src/lib.rs \
    && cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust_api_light_simple /app/server

ENV HOST=0.0.0.0
ENV PORT=3010
ENV MAX_DB_CONNECTIONS=2
ENV RUST_LOG=info

EXPOSE 3010

CMD ["/app/server"]
