# rhizo-crypt-core

Core DAG Engine - Ephemeral Working Memory for ecoPrimals Phase 2.

## Overview

RhizoCrypt is the ephemeral DAG engine that provides git-like functionality for
capturing, linking, and eventually committing events to the permanent LoamSpine layer.

## Key Features

- **Content-addressed vertices** using Blake3 hashing
- **Session-scoped DAGs** with full lifecycle management
- **Merkle tree proofs** for cryptographic verification
- **Slice semantics** with 6 modes (Copy, Loan, Consignment, Escrow, Waypoint, Transfer)
- **Dehydration protocol** for committing to LoamSpine
- **Live primal clients** for Songbird, BearDog, NestGate, LoamSpine
- **Capability-based discovery** for runtime service location
- **Multiple storage backends** with trait-based extensibility
  - In-memory (default)
  - RocksDB (optional, `--features rocksdb`)
- **Storage health & statistics** for observability
- **Zero unsafe code** - `#![forbid(unsafe_code)]`

## Usage

```rust
use rhizo_crypt_core::{
    PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder, EventType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and start the primal
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    // Create a session
    let session = SessionBuilder::new(SessionType::General)
        .with_name("My Session")
        .build();
    
    // Create vertices
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

// RocksDB (persistent, requires feature flag)
#[cfg(feature = "rocksdb")]
{
    use rhizo_crypt_core::RocksDbDagStore;
    let store = RocksDbDagStore::open("/path/to/db")?;
}
```

## Live Clients

Connect to Phase 1 primals at runtime:

```rust
use rhizo_crypt_core::clients::{SongbirdClient, BearDogClient};
use rhizo_crypt_core::discovery::DiscoveryRegistry;
use std::sync::Arc;

// Service discovery via Songbird
let songbird = SongbirdClient::from_env();
songbird.connect().await?;
songbird.register("127.0.0.1:9400").await?;

// Capability-based discovery
let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
let beardog = BearDogClient::with_discovery(registry);
beardog.connect().await?;
let sig = beardog.sign_vertex(&hash, &did).await?;
```

## Client Factory

Centralized capability-based client creation:

```rust
use rhizo_crypt_core::integration::ClientFactory;
use rhizo_crypt_core::discovery::DiscoveryRegistry;
use std::sync::Arc;

let registry = Arc::new(DiscoveryRegistry::new("rhizoCrypt"));
let factory = ClientFactory::new(registry);

// Get endpoints for different services
let signing_addr = factory.signing_endpoint().await?;
let commit_addr = factory.commit_endpoint().await?;
let storage_addr = factory.storage_endpoint().await?;
```

## Modules

| Module | Purpose |
|--------|---------|
| `types` | Core types: `VertexId`, `SessionId`, `Did`, `Timestamp` |
| `vertex` | Vertex structure with builder pattern |
| `event` | 25+ event types across 7 domains |
| `session` | Session management and lifecycle |
| `store` | DAG and payload storage traits + in-memory impl |
| `store_rocksdb` | RocksDB storage backend (optional) |
| `merkle` | Merkle trees and inclusion proofs |
| `slice` | Slice semantics with 6 modes |
| `dehydration` | Commit protocol with attestations |
| `discovery` | Capability-based runtime discovery |
| `integration` | Client traits + ClientFactory |
| `clients` | Live primal clients |
| `config` | Configuration structs |
| `error` | Error types |
| `primal` | Lifecycle and health traits |

## Feature Flags

| Feature | Description |
|---------|-------------|
| `rocksdb` | Enable RocksDB persistent storage backend |
| `test-utils` | Enable test utilities and mock implementations |

## License

AGPL-3.0
