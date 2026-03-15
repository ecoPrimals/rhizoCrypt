# rhizo-crypt-core

Core DAG Engine — Ephemeral Working Memory for ecoPrimals Phase 2.

## Overview

rhizoCrypt is the ephemeral DAG engine that provides content-addressed,
session-scoped graphs for capturing, linking, and eventually committing
events to permanent storage via dehydration.

## Key Features

- **Content-addressed vertices** using BLAKE3 hashing
- **Session-scoped DAGs** with full lifecycle management
- **Merkle tree proofs** for cryptographic verification
- **Slice semantics** with 6 modes (Copy, Loan, Consignment, Escrow, Waypoint, Transfer)
- **Dehydration protocol** for committing to permanent storage
- **Capability-based discovery** — runtime service location, zero hardcoded vendors
- **Multiple storage backends** with trait-based extensibility
  - redb (default, 100% Pure Rust)
  - sled (optional, `--features sled`)
  - In-memory (default for ephemeral sessions)
- **Storage health & statistics** for observability
- **Zero unsafe code** — `#![forbid(unsafe_code)]`

## Usage

```rust
use rhizo_crypt_core::{
    PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder, EventType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;

    let session = SessionBuilder::new(SessionType::General)
        .with_name("My Session")
        .build();

    let vertex = VertexBuilder::new(EventType::SessionStart)
        .build();

    Ok(())
}
```

## Storage Backends

```rust
use rhizo_crypt_core::store::{DagStore, InMemoryDagStore, StorageHealth};

// In-memory (default, ephemeral)
let store = InMemoryDagStore::new();

// Storage health and stats
let health = store.health(&session_id)?;  // Healthy | Degraded | Unhealthy
let stats = store.stats(&session_id)?;    // sessions, vertices, bytes, ops

// redb (persistent, Pure Rust, default backend)
#[cfg(feature = "redb")]
{
    use rhizo_crypt_core::RedbDagStore;
    let store = RedbDagStore::open("/path/to/db")?;
}

// sled (persistent, optional)
#[cfg(feature = "sled")]
{
    use rhizo_crypt_core::SledDagStore;
    let store = SledDagStore::open("/path/to/db")?;
}
```

## Capability-Based Clients

All integration uses capability discovery — no vendor lock-in:

```rust
use rhizo_crypt_core::clients::capabilities::SigningClient;
use rhizo_crypt_core::integration::ClientFactory;
use rhizo_crypt_core::discovery::DiscoveryRegistry;
use std::sync::Arc;

// Discover capabilities at runtime
let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
let factory = ClientFactory::new(registry);

// Check availability and get endpoints
if factory.has_signing_capability().await {
    let endpoint = factory.signing_endpoint().await?;
}
```

## Modules

| Module | Purpose |
|--------|---------|
| `types` | Core types: `VertexId`, `SessionId`, `Did`, `Timestamp` |
| `vertex` | Vertex structure with builder pattern |
| `event` | 25+ event types across 7 domains |
| `session` | Session management and lifecycle |
| `store` | DAG and payload storage traits + in-memory impl |
| `store_redb` | redb storage backend (default, Pure Rust) |
| `store_sled` | sled storage backend (optional) |
| `merkle` | Merkle trees and inclusion proofs |
| `slice` | Slice semantics with 6 modes |
| `dehydration` | Commit protocol with attestations |
| `discovery` | Capability-based runtime discovery |
| `integration` | Provider traits + ClientFactory |
| `clients` | Capability clients + protocol adapters |
| `config` | Configuration structs |
| `error` | Error types |
| `primal` | Lifecycle and health traits |

## Feature Flags

| Feature | Description |
|---------|-------------|
| `redb` | Enable redb persistent storage backend (default, Pure Rust) |
| `sled` | Enable sled persistent storage backend (uses libc) |
| `http-clients` | Enable HTTP clients via reqwest (pulls ring/rustls) |
| `live-clients` | Enable live connections to sibling primals (tarpc + HTTP) |
| `test-utils` | Enable test utilities and mock implementations |

## License

AGPL-3.0-or-later
