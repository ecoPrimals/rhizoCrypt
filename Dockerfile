# Multi-stage build for rhizoCrypt service
# Optimized for size and security

# Stage 1: Builder
FROM rust:1.75-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (cached layer)
RUN cargo fetch

# Build release binary
RUN cargo build --release -p rhizocrypt-service

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 rhizocrypt

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/rhizocrypt-service /app/rhizocrypt-service

# Set ownership
RUN chown -R rhizocrypt:rhizocrypt /app

# Switch to non-root user
USER rhizocrypt

# Expose RPC port
EXPOSE 9400

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD /app/rhizocrypt-service --health || exit 1

# Run the service
ENTRYPOINT ["/app/rhizocrypt-service"]
CMD []

# Labels
LABEL org.opencontainers.image.title="rhizoCrypt"
LABEL org.opencontainers.image.description="Ephemeral DAG engine for ecoPrimals"
LABEL org.opencontainers.image.version="0.12.0"
LABEL org.opencontainers.image.vendor="ecoPrimals Project"
LABEL org.opencontainers.image.licenses="AGPL-3.0"

