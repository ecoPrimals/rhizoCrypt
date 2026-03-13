# 🌱 Showcase Environment Variables

**Purpose**: Zero-hardcoding configuration for all showcase demos  
**Philosophy**: Infant discovery - services start with zero knowledge

---

## Quick Start

```bash
# Source in any showcase script:
source "$(dirname "$0")/../../showcase-env.sh"

# Or set before running:
export RHIZOCRYPT_PORT=9400
export SONGBIRD_TOWER=https://localhost:7500
./demo-script.sh
```

---

## Environment Variables

### RhizoCrypt Service

| Variable | Default | Description |
|----------|---------|-------------|
| `RHIZOCRYPT_PORT` | `0` | RPC port (0 = OS-assigned) |
| `RHIZOCRYPT_HOST` | `127.0.0.1` | RPC host |
| `RHIZOCRYPT_ENV` | `development` | Environment mode |

### Discovery (Bootstrap Adapter)

| Variable | Default | Description |
|----------|---------|-------------|
| `RHIZOCRYPT_DISCOVERY_ADAPTER` | _(none)_ | Bootstrap service endpoint |
| `SONGBIRD_TOWER` | `https://localhost:7500` | Songbird tower address |

### Capability Endpoints (Optional)

| Variable | Default | Description |
|----------|---------|-------------|
| `SIGNING_ENDPOINT` | _(none)_ | Signing capability provider |
| `PAYLOAD_STORAGE_ENDPOINT` | _(none)_ | Payload storage provider |
| `PERMANENT_STORAGE_ENDPOINT` | _(none)_ | Permanent storage provider |
| `COMPUTE_ENDPOINT` | _(none)_ | Compute orchestration provider |
| `PROVENANCE_ENDPOINT` | _(none)_ | Provenance tracking provider |

### Primal Bins

| Variable | Default | Description |
|----------|---------|-------------|
| `PRIMAL_BINS` | `../../../primalBins` | Phase 1 primal binaries |

### Timeouts

| Variable | Default | Description |
|----------|---------|-------------|
| `RHIZOCRYPT_TIMEOUT` | `30` | Connection timeout (seconds) |
| `DISCOVERY_TIMEOUT` | `10` | Discovery timeout (seconds) |

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `LOG_DIR` | `./logs` | Log directory for demos |

---

## Helper Functions

The `showcase-env.sh` script provides:

- `log(msg)` - Timestamped info message
- `success(msg)` - Success message with ✓
- `warn(msg)` - Warning message with ⚠
- `error(msg)` - Error message with ✗
- `print_env()` - Display current environment
- `check_port(port)` - Check if port is in use
- `find_available_port(base)` - Find next available port
- `wait_for_service(host, port, timeout)` - Wait for service ready

---

## Examples

### Standalone Mode (No Discovery)

```bash
# Start rhizoCrypt with OS-assigned port
export RHIZOCRYPT_PORT=0
cargo run -p rhizocrypt-service -- server
```

### With Songbird Discovery

```bash
# Start with Songbird as bootstrap
export RHIZOCRYPT_DISCOVERY_ADAPTER="https://localhost:7500"
cargo run -p rhizocrypt-service -- server
```

### With Direct Capability Endpoints

```bash
# For development/testing without discovery
export SIGNING_ENDPOINT="http://localhost:9500"
export PAYLOAD_STORAGE_ENDPOINT="http://localhost:8080"
cargo run -p rhizocrypt-service -- server
```

---

## Philosophy: Infant Discovery

1. **Birth**: Service starts with zero knowledge
2. **Self-awareness**: Knows only `RHIZOCRYPT_*` variables
3. **Bootstrap** (optional): Connects to discovery adapter
4. **Discovery**: Finds capabilities at runtime
5. **Operation**: Uses discovered services

**No hardcoding. No vendor lock-in. Pure capabilities.**

---

## Migration from Hardcoded Scripts

**Before**:
```bash
RHIZOCRYPT_PORT=9400  # Hardcoded
curl http://localhost:9400/health
```

**After**:
```bash
source showcase-env.sh  # Gets defaults from env
RHIZOCRYPT_PORT=${RHIZOCRYPT_PORT:-9400}  # Fallback
curl "http://${RHIZOCRYPT_HOST}:${RHIZOCRYPT_PORT}/health"
```

---

**Status**: All showcase demos can use this system.
**Backward Compatible**: Demos work with or without sourcing.

> For the full environment variable reference, see [docs/ENV_VARS.md](../docs/ENV_VARS.md).

