# SPDX-License-Identifier: AGPL-3.0-or-later
# Multi-stage build for rhizoCrypt service

# Stage 1: Builder
FROM rust:1.87-slim AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

RUN cargo fetch
RUN cargo build --release -p rhizocrypt-service

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 rhizocrypt

WORKDIR /app

COPY --from=builder /build/target/release/rhizocrypt /app/rhizocrypt

RUN chown -R rhizocrypt:rhizocrypt /app

USER rhizocrypt

EXPOSE 9400

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD /app/rhizocrypt status || exit 1

ENTRYPOINT ["/app/rhizocrypt"]
CMD ["server"]

LABEL org.opencontainers.image.title="rhizoCrypt"
LABEL org.opencontainers.image.description="Ephemeral DAG engine for ecoPrimals"
LABEL org.opencontainers.image.version="0.13.0-dev"
LABEL org.opencontainers.image.vendor="ecoPrimals Project"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
