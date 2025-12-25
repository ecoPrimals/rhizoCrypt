# ✅ Audit Action Checklist

**Date**: December 24, 2025  
**Status**: 2 critical, 3 high priority items

---

## 🔴 Critical (Do Now - 7 minutes)

### [ ] 1. Install libclang (5 min)

```bash
sudo apt install libclang-dev clang
```

**Verify**:
```bash
cargo clippy --workspace --all-features --all-targets -- -D warnings
```

**Expected**: 0 warnings

---

### [ ] 2. Run cargo fmt (2 min)

```bash
cd /home/strandgate/Development/ecoPrimals/phase2/rhizoCrypt
cargo fmt --all
```

**Verify**:
```bash
cargo fmt --check
```

**Expected**: No changes needed

---

## 🟡 High Priority (This Week - 8 hours)

### [ ] 3. Refactor songbird.rs (2 hours)

**Current**: 1161 lines (over 1000 limit)

**Split into**:
```
crates/rhizo-crypt-core/src/clients/songbird/
  ├── mod.rs         (re-exports)
  ├── client.rs      (~400 lines - core client)
  ├── heartbeat.rs   (~200 lines - heartbeat mechanism)
  ├── config.rs      (~150 lines - configuration)
  └── tests.rs       (~400 lines - tests)
```

**Verify**:
```bash
find crates -name '*.rs' -exec wc -l {} + | sort -nr | head -5
# All files should be < 1000 lines
```

---

### [ ] 4. Increase client test coverage (4 hours)

**Targets**:
- `toadstool.rs`: 40.30% → 80%+
- `sweetgrass.rs`: 58.20% → 80%+
- `songbird.rs`: 71.80% → 80%+

**Add**:
- Integration tests with mock HTTP
- Error path tests
- Edge case tests

**Verify**:
```bash
cargo llvm-cov --workspace --summary-only | grep -E "(toadstool|sweetgrass|songbird)"
```

---

### [ ] 5. Network partition chaos tests (2 hours)

**Add**: `crates/rhizo-crypt-core/tests/chaos/network_partitions_extended.rs`

**Tests**:
```rust
#[tokio::test]
async fn test_explicit_network_partition() {
    // Simulate network split
}

#[tokio::test]
async fn test_connection_timeout_handling() {
    // Simulate slow/hanging connections
}

#[tokio::test]
async fn test_partial_service_availability() {
    // Some services up, others down
}
```

**Verify**:
```bash
cargo test --test chaos_tests -- --nocapture
```

---

## 🟢 Medium Priority (Next Sprint - 7 hours)

### [ ] 6. Zero-copy profiling (4 hours)

**Only if profiling shows hot paths**:

```bash
cargo install cargo-flamegraph
cargo flamegraph --test e2e_tests
# Analyze flamegraph.svg for cloning hot paths
```

**If needed, optimize**:
```rust
// Option 1: Arc<Vertex> in storage
pub type VertexRef = Arc<Vertex>;

// Option 2: Cow<'a, Vertex> for read/write
pub enum VertexHandle<'a> {
    Borrowed(&'a Vertex),
    Owned(Vertex),
}
```

---

### [ ] 7. Extended documentation (3 hours)

**Add**:
1. `docs/COMMON_PITFALLS.md`
   - Session lifecycle gotchas
   - Dehydration best practices
   - Slice mode selection guide

2. `docs/PERFORMANCE_TUNING.md`
   - Benchmark interpretation
   - Configuration tuning
   - Profiling guide

3. `docs/TROUBLESHOOTING.md`
   - Common errors
   - Debug logging
   - Health check interpretation

---

## 🔵 Low Priority (Future - 5 hours)

### [ ] 8. Kubernetes deployment manifests (2 hours)

**Add**: `k8s/` directory with:
- `deployment.yaml`
- `service.yaml`
- `configmap.yaml`
- `monitoring.yaml`

---

### [ ] 9. Operational runbook (3 hours)

**Add**: `docs/OPERATIONS.md` with:
- Deployment checklist
- Monitoring dashboards
- Alerting rules
- Incident response

---

## Summary

| Priority | Items | Est. Time | Status |
|----------|-------|-----------|--------|
| 🔴 Critical | 2 | 7 min | ⏳ Pending |
| 🟡 High | 3 | 8 hours | ⏳ Pending |
| 🟢 Medium | 2 | 7 hours | ⏳ Pending |
| 🔵 Low | 2 | 5 hours | ⏳ Pending |
| **Total** | **9** | **~20 hours** | |

---

## Quick Start

**Do this now** (7 minutes):
```bash
# 1. Install libclang
sudo apt install libclang-dev clang

# 2. Run fmt
cd /home/strandgate/Development/ecoPrimals/phase2/rhizoCrypt
cargo fmt --all

# 3. Verify
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --check

# 4. Commit
git add -A
git commit -m "chore: run cargo fmt, prepare for clippy"
```

**Then** (this week):
- Refactor songbird.rs
- Increase test coverage
- Add network partition tests

---

*Generated: December 24, 2025*  
*Next review: After critical items complete*

