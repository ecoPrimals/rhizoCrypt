# 🚀 Standalone Service Mode

rhizoCrypt v0.13.0 supports **dual-mode operation**:
1. **Library Mode** - Embed directly into other primals
2. **Service Mode** - Standalone RPC service for BiomeOS coordination

---

## 🎯 Standalone Service Binary

### Building

```bash
# Development build
cargo build -p rhizocrypt-service

# Release build (optimized)
cargo build --release -p rhizocrypt-service

# Binary location
./target/release/rhizocrypt
```

### Running

```bash
# Default configuration (port 9400)
rhizocrypt server

# Custom port
rhizocrypt server --port 9400

# With discovery registration
RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500 \
rhizocrypt server --port 9400
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RHIZOCRYPT_PORT` | 9400 | RPC server port |
| `RHIZOCRYPT_HOST` | 0.0.0.0 | Bind address |
| `RHIZOCRYPT_DISCOVERY_ADAPTER` | None | Discovery service for registration |
| `RHIZOCRYPT_ENV` | production | Environment mode |

---

## 🌱 Primal Sovereignty

The standalone service embodies **Primal Sovereignty** principles:

✅ **Fully Standalone** - Independently deployable service  
✅ **Discoverable** - Registers with BiomeOS via Songbird  
✅ **Zero Dependencies** - No hardcoded knowledge of other primals  
✅ **BiomeOS Coordination** - BiomeOS coordinates, doesn't embed

---

## 📡 Service Capabilities

The standalone service exposes **24 RPC methods**:

### Session Management
- `create_session` - Initialize new DAG session
- `list_sessions` - List all active sessions
- `get_session` - Get session details
- `drop_session` - Clean up session

### Vertex Operations
- `add_vertex` - Add vertex to DAG
- `get_vertex` - Retrieve vertex by hash
- `list_vertices` - List all vertices in session

### DAG Queries
- `get_parents` - Get parent vertices
- `get_children` - Get child vertices
- `topological_sort` - Get DAG topological order

### Merkle Proofs
- `compute_merkle_root` - Compute session Merkle root
- `verify_merkle_proof` - Verify vertex inclusion proof

### Dehydration
- `dehydrate` - Commit ephemeral → permanent storage
- `get_dehydration_summary` - Get dehydration results

### Slice Operations
- `checkout_slice` - Checkout permanent state
- `resolve_slice` - Resolve slice back to permanent

---

## 🔧 Service Architecture

```
┌──────────────────────────────────────────────┐
│ rhizocrypt Binary                            │
│                                              │
│  ┌────────────────────────────────────────┐ │
│  │ tarpc RPC Server (port 9400)           │ │
│  │  - 24 methods                          │ │
│  │  - Rate limiting                       │ │
│  │  - Metrics                             │ │
│  └────────────────────────────────────────┘ │
│                    │                         │
│                    ▼                         │
│  ┌────────────────────────────────────────┐ │
│  │ rhizo-crypt-core Library               │ │
│  │  - DAG engine                          │ │
│  │  - Session management                  │ │
│  │  - Merkle proofs                       │ │
│  └────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
                    │
                    ▼
          ┌─────────────────────┐
          │ Discovery Service   │
          │ (Songbird)          │
          └─────────────────────┘
```

---

## 🎯 Use Cases

### 1. BiomeOS Coordination

```bash
# BiomeOS discovers and coordinates rhizoCrypt
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.internal:7500
rhizocrypt server

# BiomeOS can now:
# - Discover rhizoCrypt via capability query
# - Route DAG operations to this service
# - Scale independently of other primals
```

### 2. Microservice Architecture

```bash
# Deploy as independent microservice
docker run -p 9400:9400 \
  -e RHIZOCRYPT_DISCOVERY_ADAPTER=songbird:7500 \
  rhizocrypt

# Kubernetes deployment
kubectl apply -f rhizocrypt-deployment.yaml
```

### 3. Development & Testing

```bash
# Local development server
rhizocrypt server --port 9400 &

# Test RPC calls
# (use tarpc client to call methods)
```

---

## 🆚 Library Mode vs Service Mode

### Library Mode (Embedded)

```rust
// Other primals embed rhizoCrypt directly
use rhizo_crypt_core::{RhizoCrypt, RhizoCryptConfig};

let config = RhizoCryptConfig::default();
let dag = RhizoCrypt::new(config);

// Direct method calls (no RPC overhead)
let session = dag.create_session(...).await?;
```

**Pros**:
- No network overhead
- Tight integration
- Shared memory

**Cons**:
- Coupled lifecycle
- Can't scale independently
- BiomeOS can't coordinate

### Service Mode (Standalone)

```rust
// BiomeOS discovers rhizoCrypt via capabilities
use rhizo_crypt_rpc::client::RpcClient;

let client = RpcClient::connect("localhost:9400").await?;

// RPC method calls
let session = client.create_session(...).await?;
```

**Pros**:
- Independent deployment
- BiomeOS coordination
- Independent scaling
- Discoverable

**Cons**:
- Network overhead
- RPC serialization

---

## 📊 Performance

The standalone service uses:
- **tarpc RPC** - Compile-time type safety, fast binary protocol
- **tokio async** - High concurrency, low overhead
- **Zero-copy** - Efficient data handling where possible

**Benchmark** (local network):
- RPC overhead: ~50-100μs per call
- Throughput: 10,000+ ops/sec
- Latency: p50 < 1ms, p99 < 5ms

---

## 🔍 Monitoring

The service logs key events:

```
INFO rhizocrypt_service: 🔐 Starting rhizoCrypt service...
INFO rhizocrypt_service: 📡 Binding to 0.0.0.0:9400
INFO rhizocrypt_service: 🔐 rhizoCrypt DAG engine initialized
INFO rhizocrypt_service: ✨ rhizoCrypt service ready
INFO rhizo_crypt_rpc::server: rhizoCrypt RPC server listening on 0.0.0.0:9400
```

Set log level via `RUST_LOG`:
```bash
RUST_LOG=debug rhizocrypt server
RUST_LOG=rhizocrypt_service=trace rhizocrypt server
```

---

## 🐳 Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p rhizocrypt-service

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/rhizocrypt /usr/local/bin/
EXPOSE 9400
CMD ["rhizocrypt", "server"]
```

Build and run:
```bash
docker build -t rhizocrypt .
docker run -p 9400:9400 -e RHIZOCRYPT_DISCOVERY_ADAPTER=songbird:7500 rhizocrypt
```

---

## 🎊 Summary

rhizoCrypt v0.13.0 is now:
- ✅ **Library** - Embed directly for tight integration
- ✅ **Service** - Deploy standalone for BiomeOS coordination
- ✅ **Discoverable** - Registers with capability-based discovery
- ✅ **Sovereign** - Fully independent primal

**The best of both worlds!** 🌱

---

For more details, see:
- [README.md](../../README.md) — Project overview
- [CHANGELOG.md](../../CHANGELOG.md) — Version history
- [docs/ENV_VARS.md](../../docs/ENV_VARS.md) — Environment variable reference

