# 🔐 rhizoCrypt — Environment Variables

**Last Updated**: March 16, 2026  
**Version**: 0.13.0-dev  
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
| `RHIZOCRYPT_PORT` | u16 | `9400` | RPC server port |
| `RHIZOCRYPT_METRICS_PORT` | u16 | `9401` | Prometheus metrics port |

### Capability Endpoints (Preferred ✅)

These are the **new, capability-based** environment variables. Use these for all new deployments.

| Capability | Variable | Legacy Alternative | Description |
|------------|----------|-------------------|-------------|
| **Discovery** | `RHIZOCRYPT_DISCOVERY_ADAPTER` | `DISCOVERY_ENDPOINT`, `SONGBIRD_ADDRESS` | Service discovery endpoint (highest priority) |
| **Signing** | `SIGNING_ENDPOINT` | `BEARDOG_ADDRESS` | Cryptographic signing service |
| **DID Verification** | `DID_ENDPOINT` | `BEARDOG_ADDRESS` | DID resolution and verification |
| **Payload Storage** | `PAYLOAD_STORAGE_ENDPOINT` | `NESTGATE_ADDRESS` | Content-addressed payload storage |
| **Permanent Storage** | `PERMANENT_STORAGE_ENDPOINT` | `LOAMSPINE_ADDRESS` | Permanent commit storage |
| **Compute** | `COMPUTE_ENDPOINT` | `TOADSTOOL_ADDRESS` | Compute orchestration |
| **Provenance** | `PROVENANCE_ENDPOINT` | `SWEETGRASS_PUSH_ADDRESS` | Provenance tracking and queries |

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

## ⚠️ Legacy Environment Variables (Deprecated)

These variables still work for **backward compatibility** but emit deprecation warnings. **Migrate to capability-based variables above.**

| Legacy Variable | Replacement | Status |
|----------------|-------------|--------|
| `BEARDOG_ADDRESS` | `SIGNING_ENDPOINT` | ⚠️ Deprecated |
| `BEARDOG_TIMEOUT_MS` | `SIGNING_TIMEOUT_MS` | ⚠️ Deprecated |
| `NESTGATE_ADDRESS` | `PAYLOAD_STORAGE_ENDPOINT` | ⚠️ Deprecated |
| `NESTGATE_TIMEOUT_MS` | `PAYLOAD_TIMEOUT_MS` | ⚠️ Deprecated |
| `NESTGATE_MAX_PAYLOAD` | `PAYLOAD_MAX_SIZE_MB` | ⚠️ Deprecated |
| `LOAMSPINE_ADDRESS` | `PERMANENT_STORAGE_ENDPOINT` | ⚠️ Deprecated |
| `LOAMSPINE_TIMEOUT_MS` | `PERMANENT_STORAGE_TIMEOUT_MS` | ⚠️ Deprecated |
| `TOADSTOOL_ADDRESS` | `COMPUTE_ENDPOINT` | ⚠️ Deprecated |
| `TOADSTOOL_TIMEOUT_MS` | `COMPUTE_TIMEOUT_MS` | ⚠️ Deprecated |
| `SWEETGRASS_ADDRESS` | `PROVENANCE_ENDPOINT` | ⚠️ Deprecated |
| `SWEETGRASS_PUSH_ADDRESS` | `PROVENANCE_ENDPOINT` | ⚠️ Deprecated |
| `SWEETGRASS_TIMEOUT_MS` | `PROVENANCE_TIMEOUT_MS` | ⚠️ Deprecated |
| `SONGBIRD_ADDRESS` | `RHIZOCRYPT_DISCOVERY_ADAPTER` | ℹ️ Acceptable (Songbird is the universal adapter) |

---

## 📖 Usage Examples

### Development (Local Testing)

```bash
# Minimal configuration for local development
export RHIZOCRYPT_ENV=development
export DISCOVERY_ENDPOINT=localhost:8091

# rhizoCrypt will discover other capabilities through Songbird
cargo run
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

1. **Capability-based endpoint** (e.g., `SIGNING_ENDPOINT`)
2. **Legacy primal-based endpoint** (e.g., `BEARDOG_ADDRESS`) — emits warning
3. **Runtime discovery** via Songbird (if `RHIZOCRYPT_DISCOVERY_ADAPTER` or `DISCOVERY_ENDPOINT` is set)
4. **Development fallback** (only if `RHIZOCRYPT_ENV=development`)

### Example: Signing Capability

```
Priority Order:
1. SIGNING_ENDPOINT → ✅ Preferred
2. CRYPTO_SIGNING_ENDPOINT → ✅ Alternative
3. BEARDOG_ADDRESS → ⚠️ Legacy (emits warning)
4. Discover via Songbird → ✅ Runtime discovery
5. Fail gracefully → ❌ No signing available
```

---

## 🎓 Migration Guide

### Step 1: Audit Current Configuration

```bash
# Find all legacy env vars in your deployment
env | grep -E "BEARDOG|NESTGATE|LOAMSPINE|TOADSTOOL|SWEETGRASS"
```

### Step 2: Create Mapping

| Old | New |
|-----|-----|
| `BEARDOG_ADDRESS=localhost:9500` | `SIGNING_ENDPOINT=localhost:9500` |
| `NESTGATE_ADDRESS=localhost:9600` | `PAYLOAD_STORAGE_ENDPOINT=localhost:9600` |
| `LOAMSPINE_ADDRESS=localhost:9700` | `PERMANENT_STORAGE_ENDPOINT=localhost:9700` |

### Step 3: Update Configuration

```bash
# Before (deprecated):
export BEARDOG_ADDRESS=localhost:9500
export NESTGATE_ADDRESS=localhost:9600

# After (capability-based):
export SIGNING_ENDPOINT=localhost:9500
export PAYLOAD_STORAGE_ENDPOINT=localhost:9600
```

### Step 4: Verify

```bash
# Start rhizoCrypt and check logs for deprecation warnings
cargo run 2>&1 | grep -i "deprecated"

# Should see no warnings if migration is complete
```

---

## 🚀 Benefits of Capability-Based Configuration

### 1. **Primal-Agnostic**
```bash
# You don't care WHO provides signing, just that you NEED signing
SIGNING_ENDPOINT=any-signing-service:9500
```

### 2. **Flexible Deployment**
```bash
# Swap implementations without code changes
SIGNING_ENDPOINT=beardog:9500      # Use BearDog
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
- [ ] Monitor deprecation warnings in logs
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

