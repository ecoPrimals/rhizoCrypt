# 🔐 rhizoCrypt — Environment Variables

**Last Updated**: April 7, 2026  
**Version**: 0.14.0-dev  
**Philosophy**: Capability-based, not primal-based

---

## 🎯 Infant Discovery Philosophy

rhizoCrypt follows the **infant discovery** pattern: it starts with **zero knowledge** of other primals and discovers capabilities at runtime through the universal adapter (Songbird/Discovery service).

**Key Principle**: Configure by **what you need** (capability), not **who provides it** (primal name).

---

## 📋 Capability-Based Environment Variables

### Core rhizoCrypt Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RHIZOCRYPT_ENV` | string | `production` | Environment mode (`development` or `production`) |
| `RHIZOCRYPT_RPC_HOST` | string | `0.0.0.0` | RPC server bind address |
| `RHIZOCRYPT_RPC_PORT` | u16 | `9400` | tarpc server port (preferred) |
| `RHIZOCRYPT_PORT` | u16 | `9400` | tarpc server port (legacy alias for `RHIZOCRYPT_RPC_PORT`) |
| `RHIZOCRYPT_JSONRPC_PORT` | u16 | tarpc port + 1 | JSON-RPC TCP port (dual-mode: HTTP POST + newline). Defaults to tarpc port + `JSONRPC_PORT_OFFSET` (1). Set to `0` for OS-assigned. |
| `RHIZOCRYPT_METRICS_PORT` | u16 | `9401` | Prometheus metrics port |
| `XDG_RUNTIME_DIR` | path | `/run/user/$UID` | Base dir for UDS socket. When `--unix` is passed, the socket is created at `$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock`. |

### Unix Domain Socket (UDS)

rhizoCrypt supports a Unix domain socket listener for local IPC, following the
ecosystem standard from the wateringHole `IPC_COMPLIANCE_MATRIX.md`:

```bash
# Default path (ecosystem standard)
rhizocrypt server --unix
# → $XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock

# Custom path
rhizocrypt server --unix /tmp/rhizocrypt.sock
```

The UDS listener serves newline-delimited JSON-RPC 2.0 (same wire format as
`socat`, biomeOS pipeline coordinator, and other ecoPrimals tooling). The TCP
JSON-RPC port also auto-detects raw newline clients vs HTTP POST clients on a
per-connection basis.

### Capability Endpoints (Preferred ✅)

These are the **new, capability-based** environment variables. Use these for all new deployments.

| Capability | Variable | Alternative | Description |
|------------|----------|-------------|-------------|
| **Discovery** | `RHIZOCRYPT_DISCOVERY_ADAPTER` | `DISCOVERY_ENDPOINT`, `SONGBIRD_ADDRESS` | Service discovery endpoint (highest priority) |
| **Signing** | `CRYPTO_SIGNING_ENDPOINT` | `SIGNING_ENDPOINT` | Cryptographic signing service |
| **DID Verification** | `DID_VERIFICATION_ENDPOINT` | `DID_ENDPOINT` | DID resolution and verification |
| **Payload Storage** | `PAYLOAD_STORAGE_ENDPOINT` | `PAYLOAD_ENDPOINT` | Content-addressed payload storage |
| **Permanent Storage** | `STORAGE_PERMANENT_COMMIT_ENDPOINT` | `PERMANENT_STORAGE_ENDPOINT` | Permanent commit storage |
| **Compute** | `COMPUTE_ORCHESTRATION_ENDPOINT` | `COMPUTE_ENDPOINT` | Compute orchestration |
| **Provenance** | `PROVENANCE_QUERY_ENDPOINT` | `PROVENANCE_ENDPOINT` | Provenance tracking and queries |

### Capability Timeouts

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `SIGNING_TIMEOUT_MS` | u64 | `5000` | Signing operation timeout |
| `DID_TIMEOUT_MS` | u64 | `5000` | DID verification timeout |
| `PAYLOAD_TIMEOUT_MS` | u64 | `30000` | Payload storage timeout (larger for uploads) |
| `PERMANENT_STORAGE_TIMEOUT_MS` | u64 | `10000` | Permanent storage commit timeout |
| `COMPUTE_TIMEOUT_MS` | u64 | `5000` | Compute orchestration timeout |
| `PROVENANCE_TIMEOUT_MS` | u64 | `5000` | Provenance query timeout |

### Capability-Specific Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `PAYLOAD_MAX_SIZE_MB` | usize | `100` | Maximum payload size in megabytes |

---

## Legacy Environment Variables (Removed)

Legacy vendor-specific env vars (`BEARDOG_ADDRESS`, `NESTGATE_ADDRESS`, `LOAMSPINE_ADDRESS`) were
removed in v0.14.0-dev. A primal only has self-knowledge and discovers capabilities at
runtime. Use the capability-based variables above or discovery via `RHIZOCRYPT_DISCOVERY_ADAPTER`.

`SONGBIRD_ADDRESS` is still accepted as a discovery fallback (Songbird is the universal adapter).

---

## 📖 Usage Examples

### Development (Local Testing)

```bash
# Minimal configuration for local development
export RHIZOCRYPT_ENV=development
export DISCOVERY_ENDPOINT=localhost:8091

# rhizoCrypt will discover other capabilities through Songbird
cargo run -p rhizocrypt-service -- server
```

### Production (Kubernetes)

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: rhizocrypt-config
data:
  RHIZOCRYPT_ENV: "production"
  RHIZOCRYPT_RPC_HOST: "0.0.0.0"
  RHIZOCRYPT_RPC_PORT: "9400"
  
  # Capability endpoints (discovered via service mesh)
  DISCOVERY_ENDPOINT: "discovery-service.default.svc.cluster.local:8091"
  SIGNING_ENDPOINT: "signing-service.default.svc.cluster.local:9500"
  PAYLOAD_STORAGE_ENDPOINT: "storage-service.default.svc.cluster.local:9600"
  PERMANENT_STORAGE_ENDPOINT: "permanent-storage.default.svc.cluster.local:9700"
  COMPUTE_ENDPOINT: "compute-service.default.svc.cluster.local:9800"
  PROVENANCE_ENDPOINT: "provenance-service.default.svc.cluster.local:9900"
```

### Docker Compose

```yaml
version: '3.8'
services:
  rhizocrypt:
    image: rhizocrypt:latest
    environment:
      - RHIZOCRYPT_ENV=production
      - RHIZOCRYPT_RPC_PORT=9400
      - DISCOVERY_ENDPOINT=discovery:8091
      - SIGNING_ENDPOINT=signing:9500
      - PAYLOAD_STORAGE_ENDPOINT=storage:9600
      - PERMANENT_STORAGE_ENDPOINT=permanent:9700
    ports:
      - "9400:9400"
```

---

## 🔍 Discovery Priority

For each capability, rhizoCrypt checks environment variables in this order:

1. **Capability-based endpoint** (e.g., `CRYPTO_SIGNING_ENDPOINT`)
2. **Short-form endpoint** (e.g., `SIGNING_ENDPOINT`)
3. **Runtime discovery** via Songbird (if `RHIZOCRYPT_DISCOVERY_ADAPTER` or `DISCOVERY_ENDPOINT` is set)
4. **Development fallback** (only if `RHIZOCRYPT_ENV=development`)

### Example: Signing Capability

```
Priority Order:
1. CRYPTO_SIGNING_ENDPOINT → ✅ Preferred
2. SIGNING_ENDPOINT → ✅ Short form
3. Discover via Songbird → ✅ Runtime discovery
4. Fail gracefully → ❌ No signing available
```

---

## 🎓 Migration Guide

### Capability-Based Configuration

```bash
# Capability-based (recommended):
export SIGNING_ENDPOINT=localhost:9500
export PAYLOAD_STORAGE_ENDPOINT=localhost:9600
export PERMANENT_STORAGE_ENDPOINT=localhost:9700

# Or just use discovery (all capabilities resolved at runtime):
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500
```

---

## Capability-Based Configuration Benefits

### 1. **Primal-Agnostic**
```bash
# You don't care WHO provides signing, just that you NEED signing
SIGNING_ENDPOINT=any-signing-service:9500
```

### 2. **Flexible Deployment**
```bash
# Swap implementations without code changes
SIGNING_ENDPOINT=signer:9500       # Use signing provider
SIGNING_ENDPOINT=hsm-service:9500  # Use HSM service
SIGNING_ENDPOINT=kms-proxy:9500    # Use cloud KMS
```

### 3. **Easier Testing**
```bash
# Point to mock services for testing
SIGNING_ENDPOINT=localhost:19500
PAYLOAD_STORAGE_ENDPOINT=localhost:19600
```

### 4. **Ecosystem Evolution**
```bash
# New primals integrate seamlessly - no hardcoding
SIGNING_ENDPOINT=new-signing-primal:9500
```

---

## 🔒 Security Considerations

### Production Checklist

- [ ] Set `RHIZOCRYPT_ENV=production` (disables development fallbacks)
- [ ] Configure `DISCOVERY_ENDPOINT` for runtime discovery
- [ ] Use TLS for all capability endpoints
- [ ] Restrict network access to capability services
- [ ] Rotate capability endpoints without code changes

### Secrets Management

```bash
# Use secret management for sensitive endpoints
SIGNING_ENDPOINT=$(vault kv get -field=endpoint secret/signing)
PERMANENT_STORAGE_ENDPOINT=$(kubectl get secret storage-endpoint -o jsonpath='{.data.endpoint}' | base64 -d)
```

---

## Build Environment

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `CARGO_TARGET_DIR` | path | (from `.cargo/config.toml`) | Build artifact directory. Set explicitly if global `~/.cargo/config.toml` overrides the project-local config (e.g. when global target-dir is on a noexec mount). |

```bash
export CARGO_TARGET_DIR="$HOME/.cargo-build/rhizoCrypt/target"
```

---

## Related Documentation

- [specs/ARCHITECTURE.md](../specs/ARCHITECTURE.md) — Primal-agnostic design
- [DEPLOYMENT_CHECKLIST.md](./DEPLOYMENT_CHECKLIST.md) — Production deployment guide

---

*"Configure by capability, not by name. Discover, don't hardcode."* 🐣

