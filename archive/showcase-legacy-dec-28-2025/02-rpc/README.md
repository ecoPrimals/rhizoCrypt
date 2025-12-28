# 🔐 rhizoCrypt RPC Layer Demos

**Phase 2: RPC Layer — tarpc Server/Client**

---

## 📋 What You'll Learn

- Starting the tarpc RPC server
- All 24 RPC methods
- Rate limiting (token bucket)
- Prometheus-compatible metrics
- Graceful shutdown

---

## 🚀 Quick Start

```bash
# Terminal 1: Start the server
cd server && ./start-server.sh

# Terminal 2: Run client demos
cd client && ./demo-rpc-client.sh
```

---

## 📁 Available Demos

### `server/start-server.sh`
**Time:** 2 minutes  
**Complexity:** Beginner

Starts the tarpc RPC server with:
- Configurable port (default: 9400)
- Rate limiting enabled
- Metrics endpoint on :9401

### `client/demo-rpc-client.sh`
**Time:** 10 minutes  
**Complexity:** Intermediate

Demonstrates all 24 RPC methods:
- Session management (create, get, resolve, discard)
- Vertex operations (append, get, list)
- DAG queries (frontier, genesis, children)
- Merkle operations (root, proof, verify)
- Slice operations (create, checkout, return, cancel)

### `metrics/demo-metrics.sh`
**Time:** 5 minutes  
**Complexity:** Beginner

Shows Prometheus-compatible metrics:
- Request counts
- Latency histograms
- Error rates
- Rate limit rejections

---

## 🔧 RPC Methods (24 Total)

### Session Management (4)
| Method | Description |
|--------|-------------|
| `create_session` | Create new session |
| `get_session` | Get session info |
| `resolve_session` | Commit session |
| `discard_session` | Discard session |

### Vertex Operations (4)
| Method | Description |
|--------|-------------|
| `append_vertex` | Add vertex to session |
| `get_vertex` | Get vertex by ID |
| `list_vertices` | List session vertices |
| `count_vertices` | Count session vertices |

### DAG Queries (4)
| Method | Description |
|--------|-------------|
| `get_frontier` | Get leaf vertices |
| `get_genesis` | Get root vertex |
| `get_parents` | Get vertex parents |
| `get_children` | Get vertex children |

### Merkle Operations (4)
| Method | Description |
|--------|-------------|
| `get_merkle_root` | Get session Merkle root |
| `generate_proof` | Generate inclusion proof |
| `verify_proof` | Verify inclusion proof |
| `get_merkle_tree` | Get full tree |

### Slice Operations (4)
| Method | Description |
|--------|-------------|
| `create_slice` | Create data slice |
| `checkout_slice` | Check out slice |
| `return_slice` | Return slice |
| `cancel_slice` | Cancel slice |

### Primal Operations (4)
| Method | Description |
|--------|-------------|
| `health_check` | Check primal health |
| `get_metrics` | Get primal metrics |
| `dehydrate` | Trigger dehydration |
| `get_status` | Get primal status |

---

## 📊 Rate Limiting

Token bucket algorithm with defaults:
- **Capacity**: 1000 tokens
- **Refill rate**: 100 tokens/sec
- **Cost per request**: 1 token

When exhausted, requests return `RateLimited` error.

---

## 📈 Metrics

Prometheus-compatible metrics on `:9401/metrics`:

```
# Request counts by method
rhizocrypt_requests_total{method="append_vertex"} 42

# Latency histogram
rhizocrypt_request_duration_seconds_bucket{le="0.001"} 100

# Error rates
rhizocrypt_errors_total{type="rate_limited"} 5
```

---

## 🛠️ Configuration

Environment variables:
```bash
export RHIZOCRYPT_RPC_PORT=9400
export RHIZOCRYPT_METRICS_PORT=9401
export RHIZOCRYPT_RATE_LIMIT_CAPACITY=1000
export RHIZOCRYPT_RATE_LIMIT_REFILL=100
```

---

## 💡 Key Concepts

### Pure Rust RPC (tarpc)
No protobuf, no code generation, no external dependencies. Just Rust types.

### Async by Default
All RPC methods are async. The server uses tokio for concurrency.

### Graceful Shutdown
Server handles SIGTERM/SIGINT for clean shutdown.

---

## 🔗 Next Steps

After understanding RPC:
1. Explore `../03-inter-primal/` for primal integration
2. Try `../04-complete-workflow/` for full dehydration
3. See metrics in `metrics/`

---

*rhizoCrypt: Pure Rust RPC with rate limiting and metrics*

