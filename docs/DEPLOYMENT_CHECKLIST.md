# DEPLOYMENT CHECKLIST — rhizoCrypt v0.14.0-dev

**Date**: April 1, 2026
**Version**: 0.14.0-dev
**Status**: PRODUCTION READY

---

## PRE-DEPLOYMENT VERIFICATION

### Code Quality
- [x] **1,423 tests passing** (all features), 0 failures
- [x] **`--fail-under-lines 90` CI gate** enforced
- [x] **Zero unsafe code** (workspace `unsafe_code = "deny"`, zero `unsafe` in tests via temp-env)
- [x] **Zero clippy warnings** (pedantic + nursery + cargo + cast lints, `unwrap_used`/`expect_used = "deny"`, `missing_errors_doc = "warn"`)
- [x] **100% file size compliance** (all files under 1000 lines)
- [x] **Formatted** (`cargo fmt --check` clean)
- [x] **AGPL-3.0-or-later** SPDX header on all 129 `.rs` files

### Architecture
- [x] **Capability-based** (zero hardcoded primal names in production)
- [x] **Lock-free concurrency** (DashMap, atomics, lock-free CircuitBreaker)
- [x] **Dehydration protocol** (complete 7-step workflow)
- [x] **UniBin binary** (`rhizocrypt server`, `doctor`, `status`, `version`)
- [x] **JSON-RPC 2.0 + tarpc** — dual-transport, semantic method names
- [x] **Graceful shutdown** (SIGTERM + SIGINT, no data loss)

### Storage Backends
- [x] **redb** (default) — 100% Pure Rust, ACID, MVCC, ecoBin compliant
- [x] **Memory** (testing) — ephemeral in-memory store

### Documentation
- [x] **README.md** (current metrics — 1,423 tests)
- [x] **CHANGELOG.md** (version history through session 26)
- [x] **showcase/** (70+ comprehensive demos)
- [x] **specs/** (9 complete + 1 experimental specification documents)
- [x] **docs/ENV_VARS.md** (capability-based configuration reference)

---

## DEPLOYMENT OPTIONS

### Option 1: Standalone Binary

```bash
# Build release binary
cargo build --release -p rhizocrypt-service

# Run service (production port)
./target/release/rhizocrypt server --port 9400

# Health check via JSON-RPC
curl -s -X POST http://localhost:9400/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}'
```

**Environment Variables**:
```bash
export RHIZOCRYPT_PORT=9400
export RHIZOCRYPT_ENV=production
export RHIZOCRYPT_LOG_LEVEL=info
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500  # Optional: for registration
```

---

### Option 2: Docker Container (musl-static + Alpine)

The Dockerfile produces a musl-static binary in a multi-stage build (ecoBin compliant).
Runtime image is Alpine 3.20 with a non-root user (UID 1000).

```bash
# Build Docker image (multi-stage musl-static)
docker build -t rhizocrypt:0.14.0 .

# Run container
docker run -d \
  --name rhizocrypt \
  -p 9400:9400 \
  -e RHIZOCRYPT_ENV=production \
  rhizocrypt:0.14.0

# Health check via rhizocrypt status subcommand
docker exec rhizocrypt /app/rhizocrypt status
```

**Docker Compose**:
```yaml
version: '3.8'
services:
  rhizocrypt:
    image: rhizocrypt:0.14.0
    ports:
      - "9400:9400"
    environment:
      - RHIZOCRYPT_ENV=production
      - RHIZOCRYPT_LOG_LEVEL=info
    healthcheck:
      test: ["CMD", "/app/rhizocrypt", "status"]
      interval: 30s
      timeout: 10s
      retries: 3
```

---

### Option 3: Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rhizocrypt
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rhizocrypt
  template:
    metadata:
      labels:
        app: rhizocrypt
    spec:
      containers:
        - name: rhizocrypt
          image: rhizocrypt:0.14.0
          ports:
            - containerPort: 9400
          env:
            - name: RHIZOCRYPT_ENV
              value: "production"
            - name: RHIZOCRYPT_DISCOVERY_ADAPTER
              valueFrom:
                configMapKeyRef:
                  name: rhizocrypt-config
                  key: discovery_adapter
          livenessProbe:
            exec:
              command: ["/app/rhizocrypt", "status"]
            initialDelaySeconds: 5
            periodSeconds: 30
```

---

## POST-DEPLOYMENT VERIFICATION

### Health Check (JSON-RPC)

```bash
curl -s -X POST http://localhost:9400/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}'
```

### Metrics (JSON-RPC)

```bash
curl -s -X POST http://localhost:9400/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"health.metrics","params":{},"id":1}'
```

### Doctor (CLI)

```bash
# Basic diagnostics
./target/release/rhizocrypt doctor

# Comprehensive (includes connectivity probes)
./target/release/rhizocrypt doctor --comprehensive
```

---

## CONFIGURATION

### Required
- `RHIZOCRYPT_PORT` — RPC server port (default: OS-assigned in dev, 9400 in production)
- `RHIZOCRYPT_ENV` — Environment (`development` or `production`)

### Optional
- `RHIZOCRYPT_DISCOVERY_ADAPTER` — Discovery service endpoint (primary)
- `RHIZOCRYPT_LOG_LEVEL` — Logging level (default: `info`)
- `SIGNING_ENDPOINT` — Direct signing provider
- `PERMANENT_STORAGE_ENDPOINT` — Direct permanent storage
- `PAYLOAD_STORAGE_ENDPOINT` — Direct payload storage

See [ENV_VARS.md](./ENV_VARS.md) for the complete capability-based configuration reference.

---

## OPERATIONAL GUIDELINES

### Monitoring

**Key Metrics** (via `health.metrics` JSON-RPC):
- `rhizocrypt_sessions_active` — currently active sessions
- `rhizocrypt_rpc_errors_total` — errors by type
- `rhizocrypt_rpc_request_duration_seconds_mean` — latency by method
- `rhizocrypt_uptime_seconds` — time since start

### Scaling

**Horizontal Scaling** (recommended):
- Each rhizoCrypt instance is independent (no shared state)
- Load balance at discovery layer
- Sessions are scoped to the instance that created them

### Backup & Recovery

**Ephemeral Sessions**: rhizoCrypt is designed to forget. Sessions are temporary
working memory — no backup needed for in-memory state.

**Dehydrated Sessions**: Committed results live in permanent storage (via
`PermanentStorageProvider` capability). Recovery is via slice checkout.

---

## TROUBLESHOOTING

**1. Port Already in Use**
```bash
lsof -i :9400
export RHIZOCRYPT_PORT=9401
```

**2. Discovery Unavailable**
```bash
# rhizoCrypt runs standalone without discovery — just no inter-primal features
unset RHIZOCRYPT_DISCOVERY_ADAPTER
```

**3. Doctor Diagnostics**
```bash
./target/release/rhizocrypt doctor --comprehensive
```

---

## REFERENCES

- [README.md](../README.md) — Project overview
- [CHANGELOG.md](../CHANGELOG.md) — Version history
- [ENV_VARS.md](./ENV_VARS.md) — Environment variable reference
- [specs/](../specs/) — Formal specifications
- [showcase/](../showcase/) — 70+ progressive demos

---

**Created**: December 27, 2025
**Last Updated**: April 7, 2026
**Version**: rhizoCrypt 0.14.0-dev
