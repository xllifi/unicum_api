FROM rust:1.96-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install --no-install-recommends -y libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release \
    && cp target/release/unicum_api /usr/local/bin/unicum_api

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system unicum-api \
    && useradd --system --gid unicum-api --no-create-home --home-dir /nonexistent unicum-api

COPY --from=builder /usr/local/bin/unicum_api /usr/local/bin/unicum_api

USER unicum-api

EXPOSE 3000

ENTRYPOINT ["unicum_api"]
CMD ["serve", "--ip", "0.0.0.0", "--port", "3000"]
