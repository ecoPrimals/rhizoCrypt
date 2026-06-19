# rhizocrypt-service

Standalone RPC service for rhizoCrypt v0.14.17 — Ephemeral DAG Engine.

## Dual-Mode Operation

1. **Library Mode** — Embed `rhizo-crypt-core` directly into other primals
2. **Service Mode** — Standalone UniBin binary for biomeOS coordination

---

## Running

```bash
# Development (UDS-only, default)
cargo run -p rhizocrypt-service -- server

# Development with TCP opt-in
cargo run -p rhizocrypt-service -- server --port 9400

# Production (plasmidBin-built binary)
rhizocrypt server --socket /run/biomeos/rhizocrypt.sock
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RHIZOCRYPT_PORT` | *(none)* | TCP port (opt-in) |
| `RHIZOCRYPT_HOST` | `0.0.0.0` | Bind address |
| `RHIZOCRYPT_ENV` | `production` | Environment mode |
| `DISCOVERY_ENDPOINT` | *(none)* | Discovery adapter for registration |
| `FAMILY_ID` | *(none)* | BTSP family-scoped socket naming |
| `BIOMEOS_INSECURE` | *(none)* | Set `1` to bypass BTSP handshake (dev only) |

See [docs/ENV_VARS.md](../../docs/ENV_VARS.md) for the full reference.

---

## Transport Model

- **UDS**: Unconditional on Unix (`$XDG_RUNTIME_DIR/biomeos/rhizocrypt.sock`)
- **TCP**: Opt-in via `--port` or env vars (tarpc + JSON-RPC dual-mode)
- **BTSP Phase 3**: X25519 handshake + ChaCha20-Poly1305 encrypted channel
- **JSON-RPC 2.0**: 37 methods across 7 domains, HTTP + newline-delimited

## Subcommands

| Command | Description |
|---------|-------------|
| `server` | Start the RPC service |
| `client health` | Check server health |
| `client list-sessions` | List active sessions |
| `client metrics` | Get service metrics |
| `doctor` | Run diagnostics |
| `doctor --comprehensive` | Include connectivity probes |
| `status` | Print connection status |
| `version` | Print version info |

---

## Docker (musl-static + scratch)

```bash
docker build -t rhizocrypt:0.14.17 .
docker run -d --name rhizocrypt -p 9400:9400 rhizocrypt:0.14.17
```

See root `Dockerfile` for the multi-stage musl-static build (ecoBin compliant,
`FROM scratch`, non-root UID 1000).

---

## License

AGPL-3.0-or-later
