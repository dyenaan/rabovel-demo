FROM rust:1.98-slim-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
COPY crates ./crates
COPY migrations ./migrations

# --locked: fail the build if Cargo.lock is out of date rather than silently
# re-resolving dependencies, so the image runs exactly the versions tested.
# This image only ships the HTTP API binary.
RUN cargo build -p rabovel-api --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rabovel-api /app/rabovel-api

EXPOSE 8080

CMD ["./rabovel-api"]
