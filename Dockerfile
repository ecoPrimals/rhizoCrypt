# SPDX-License-Identifier: AGPL-3.0-or-later
# Multi-stage musl-static build for rhizoCrypt service

# Stage 1: Builder (musl-static, ecoBin compliant)
FROM rust:1.87-slim AS builder

RUN rustup target add x86_64-unknown-linux-musl && \
    apt-get update && apt-get install -y musl-tools && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

RUN cargo fetch
RUN cargo build --release --target x86_64-unknown-linux-musl -p rhizocrypt-service

# Stage 2: Minimal runtime (scratch — fully static binary needs no OS)
FROM scratch

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/rhizocrypt /rhizocrypt

USER 1000:1000

EXPOSE 9400

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/rhizocrypt", "status"]

ENTRYPOINT ["/rhizocrypt"]
CMD ["server"]

LABEL org.opencontainers.image.title="rhizoCrypt"
LABEL org.opencontainers.image.description="Ephemeral DAG engine for ecoPrimals"
LABEL org.opencontainers.image.version="0.14.0-dev"
LABEL org.opencontainers.image.vendor="ecoPrimals Project"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
