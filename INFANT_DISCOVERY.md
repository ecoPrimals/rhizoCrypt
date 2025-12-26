# 🌱 Infant Discovery

**Starting with Zero Knowledge, Discovering at Runtime**

---

## 🎯 Overview

**Infant Discovery** is the principle that primals (services) in the ecoPrimals ecosystem start with **zero hardcoded knowledge** about other services and discover capabilities at runtime.

Like an infant learning about the world, primals:
1. **Wake up** with only self-awareness
2. **Find** the universal adapter (discovery service)
3. **Query** for capabilities they need
4. **Connect** to providers on-demand
5. **Adapt** as services come and go

---

## 🚀 Why Infant Discovery?

### **Problems with Hardcoding**

```rust
// ❌ OLD WAY: Hardcoded vendor lock-in
const BEARDOG_URL: &str = "http://beardog.local:9500";
const NESTGATE_URL: &str = "http://nestgate.local:8080";
const LOAMSPINE_URL: &str = "http://loamspine.local:8081";

// What if BearDog is down?
// What if you want to use YubiKey instead?
// What if services move?
// LOCKED IN! 🔒
```

### **Benefits of Infant Discovery**

```rust
// ✅ NEW WAY: Capability-based discovery
let signer = SigningClient::discover(&registry).await?;
// Works with: BearDog, YubiKey, CloudKMS, HSM, TPM, etc.
// No code changes to swap providers!
// Resilient to service changes!
// FLEXIBLE! 🌱
```

**Key Benefits**:
- ✅ **Vendor Neutrality** — No lock-in to specific providers
- ✅ **Resilience** — Automatic failover if services go down
- ✅ **Federation** — Multiple providers for same capability
- ✅ **Flexibility** — Easy testing, staging, production configs
- ✅ **Zero Coupling** — Services don't know about each other

---

## 🏗️ How It Works

### **1. Bootstrap Phase**

Primal starts with ONE piece of information (or zero!):

```bash
# Only ONE environment variable needed
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500
```

Or in **pure infant mode** (zero config):
```bash
# NO environment variables!
# Discover via multicast, DHT, mDNS, etc.
```

### **2. Discovery Phase**

Primal queries the universal adapter for capabilities:

```rust
use rhizo_crypt_core::discovery::{DiscoveryRegistry, Capability};

// Initialize discovery (finds universal adapter)
let registry = DiscoveryRegistry::new().await?;

// Query: "Who can sign things?"
let endpoints = registry.discover(&Capability::Signing).await;

// Returns ANY provider offering signing:
// - BearDog (DID-based signing)
// - YubiKey (hardware security)
// - CloudKMS (cloud HSM)
// - Local TPM (trusted platform module)
```

### **3. Connection Phase**

Create clients that work with ANY provider:

```rust
use rhizo_crypt_core::clients::capabilities::SigningClient;

// Discover and connect to ANY signing provider
let signer = SigningClient::discover(&registry).await?;

// Use it (same API regardless of provider)
let signature = signer.sign(data, &did).await?;
```

### **4. Runtime Adaptation**

Services can change without code changes:

```bash
# Development: Use local mock
export SIGNING_ENDPOINT=http://localhost:9500

# Staging: Use staging BearDog
export SIGNING_ENDPOINT=http://beardog-staging.internal:9500

# Production: Use CloudKMS
export SIGNING_ENDPOINT=https://cloudkms.googleapis.com/v1/projects/...

# OR: Let discovery find the best one
# (no environment variable needed!)
```

---

## 📊 Discovery Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: Primal Wakes Up (Zero Knowledge)                        │
├─────────────────────────────────────────────────────────────────┤
│ • No hardcoded services                                          │
│ • No hardcoded addresses                                         │
│ • Only knows: self-identity                                      │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 2: Find Universal Adapter                                   │
├─────────────────────────────────────────────────────────────────┤
│ Priority:                                                        │
│ 1. RHIZOCRYPT_DISCOVERY_ADAPTER env var                         │
│ 2. Multicast discovery (mDNS, SSDP)                             │
│ 3. DHT bootstrap nodes                                           │
│ 4. Peer exchange                                                 │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 3: Query Capabilities                                       │
├─────────────────────────────────────────────────────────────────┤
│ registry.discover(&Capability::Signing)                          │
│ registry.discover(&Capability::PayloadStorage)                   │
│ registry.discover(&Capability::PermanentCommit)                  │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 4: Select Provider                                          │
├─────────────────────────────────────────────────────────────────┤
│ Multiple providers found? Pick best:                             │
│ • Lowest latency                                                 │
│ • Highest trust score                                            │
│ • Geographic proximity                                           │
│ • Load balancing                                                 │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 5: Connect & Use                                            │
├─────────────────────────────────────────────────────────────────┤
│ let signer = SigningClient::discover(&registry).await?;          │
│ let signature = signer.sign(data, &did).await?;                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🎓 Core Concepts

### **Capabilities vs Vendors**

**Old Thinking (Vendor-Centric)**:
```
"I need to call BearDog to sign this."
→ Hardcoded dependency on BearDog
→ Locked in
```

**New Thinking (Capability-Centric)**:
```
"I need something that can sign this."
→ Query for signing capability
→ ANY provider works
→ Freedom!
```

### **Universal Adapter**

The **only** service that may be configured (or discovered):

```bash
# The ONE variable you might need
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500

# OR use legacy name (still works, with info log)
export SONGBIRD_ADDRESS=songbird.local:7500

# OR discover it (true infant mode)
# (no configuration at all!)
```

The universal adapter (e.g., Songbird) knows about all services and their capabilities. It's like a phone book or DNS for the ecosystem.

### **Federation**

Multiple providers for same capability:

```rust
// Get ALL signing providers
let endpoints = registry.discover_all(&Capability::Signing).await?;

// Choose based on criteria:
// - Primary in same region
// - Fallback in different region
// - Load balance across multiple

let primary = select_by_latency(endpoints);
let fallback = select_by_region(endpoints, "us-west");
```

---

## 🛠️ Configuration

### **Production (Minimal)**

```bash
# Only ONE variable!
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.prod.internal:7500

# Everything else discovered automatically
```

### **Development (Direct Config)**

```bash
# Bypass discovery for faster dev iteration
export SIGNING_ENDPOINT=http://localhost:9500
export PAYLOAD_STORAGE_ENDPOINT=http://localhost:8080
export PERMANENT_STORAGE_ENDPOINT=http://localhost:8081

# Still works! Legacy compatibility
export BEARDOG_ADDRESS=localhost:9500  # Emits deprecation warning
```

### **Testing (Zero Config)**

```rust
// Use mocks - no real services needed!
let factory = MockCapabilityFactory::permissive();
let signer = factory.signing_client().await?;

// Test your logic without network calls
let sig = signer.sign(test_data, &test_did).await?;
```

---

## 📝 Migration Guide

### **From v0.10.x to v0.11.x**

**Step 1**: Update environment variables (OPTIONAL - old ones still work!)

```bash
# Old (still works, shows deprecation warnings)
export BEARDOG_ADDRESS=beardog.local:9500
export NESTGATE_ADDRESS=nestgate.local:8080

# New (recommended)
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.local:7500
# OR for dev/testing:
export SIGNING_ENDPOINT=http://localhost:9500
export PAYLOAD_STORAGE_ENDPOINT=http://localhost:8080
```

**Step 2**: Update code (OPTIONAL - old clients still work!)

```rust
// Old (still works, shows deprecation warnings)
use rhizo_crypt_core::clients::BearDogClient;
let client = BearDogClient::new(config);

// New (recommended)
use rhizo_crypt_core::clients::capabilities::SigningClient;
let signer = SigningClient::discover(&registry).await?;
```

**Step 3**: Test

```bash
# All 271 tests should pass
cargo test --workspace

# Run your integration tests
# Both old and new APIs work simultaneously!
```

---

## 🎯 Use Cases

### **Multi-Cloud Deployment**

```rust
// Production: Use CloudKMS
export SIGNING_ENDPOINT=https://cloudkms.googleapis.com/...

// DR Site: Use Azure Key Vault
export SIGNING_ENDPOINT=https://keyvault.azure.net/...

// Same code, different providers!
```

### **Hardware Security**

```rust
// Use YubiKey for signing
export SIGNING_ENDPOINT=http://localhost:8080  # YubiKey HTTP bridge

// OR use TPM
export SIGNING_ENDPOINT=tpm://local  # TPM adapter

// OR use HSM
export SIGNING_ENDPOINT=pkcs11://slot0  # PKCS#11 adapter
```

### **Development Workflow**

```rust
// Local dev: Use mocks
let signer = MockSigningClient::permissive();

// Integration testing: Use real staging services
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird-staging.local:7500

// Production: Use real production services
export RHIZOCRYPT_DISCOVERY_ADAPTER=songbird.prod.internal:7500
```

---

## 🔬 Advanced Topics

### **Custom Discovery Strategies**

Implement your own discovery mechanism:

```rust
// Consul discovery
let consul_adapter = ConsulDiscoveryAdapter::new("consul.local:8500");

// Kubernetes discovery
let k8s_adapter = K8sDiscoveryAdapter::from_in_cluster_config();

// Static file discovery
let file_adapter = FileDiscoveryAdapter::from_path("/etc/services.json");
```

### **Health Checking**

```rust
// Automatic health checks
let signer = SigningClient::discover(&registry).await?;

if !signer.is_healthy().await {
    // Rediscover and reconnect
    let signer = SigningClient::discover(&registry).await?;
}
```

### **Caching & Performance**

```rust
// Factory caches clients automatically
let factory = CapabilityClientFactory::new(registry);

// First call: discovers and connects
let signer = factory.signing_client().await?;

// Subsequent calls: reuses cached client
let signer2 = factory.signing_client().await?;  // Same instance!
```

---

## 📚 API Reference

### **Discovery Registry**

```rust
use rhizo_crypt_core::discovery::{DiscoveryRegistry, Capability};

// Initialize (finds universal adapter)
let registry = DiscoveryRegistry::new().await?;

// Discover single provider
let endpoint = registry.discover(&Capability::Signing).await;

// Discover all providers
let endpoints = registry.discover_all(&Capability::Signing).await?;
```

### **Capability Clients**

```rust
use rhizo_crypt_core::clients::capabilities::*;

// Signing
let signer = SigningClient::discover(&registry).await?;
let sig = signer.sign(data, &did).await?;

// Storage
let storage = StorageClient::discover(&registry).await?;
let payload_ref = storage.put_payload(data).await?;

// Permanent Storage
let permanent = PermanentStorageClient::discover(&registry).await?;
let commit_ref = permanent.commit(&summary).await?;
```

### **Client Factory**

```rust
use rhizo_crypt_core::clients::CapabilityClientFactory;

// Create factory (auto-discovers services)
let factory = CapabilityClientFactory::new(registry);

// Get clients (lazy, cached)
let signer = factory.signing_client().await?;
let storage = factory.storage_client().await?;
```

---

## 🎉 Summary

**Infant Discovery** enables:
- ✅ **Zero Hardcoding** — No vendor lock-in
- ✅ **Runtime Flexibility** — Swap providers without code changes
- ✅ **Resilience** — Automatic failover and load balancing
- ✅ **Federation** — Multiple providers, no single points of failure
- ✅ **Testing** — Easy mocks, no network required

**Configuration Simplicity**:
- **Before**: 6+ environment variables
- **After**: 1 variable (or zero!)

**The Future is Capability-Based** 🌱

---

*"Like an infant, we start with zero knowledge and discover the world around us."*

For more details, see:
- [README.md](README.md) - Main project documentation
- [ZERO_HARDCODING_COMPLETE.md](ZERO_HARDCODING_COMPLETE.md) - Phase 1 completion report
- [STATUS.md](STATUS.md) - Current project status

