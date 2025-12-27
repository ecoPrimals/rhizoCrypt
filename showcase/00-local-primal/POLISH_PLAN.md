# 🎯 Local Showcase Polish Plan

**Status**: In Progress  
**Approach**: Apply validated API patterns from RootPulse showcase  
**Philosophy**: Real code, no mocks, expose any gaps

---

## ✅ Validated API Patterns (from RootPulse)

### Session Creation
```rust
use rhizo_crypt_core::{session::SessionBuilder, SessionType, Did};

let session = SessionBuilder::new(SessionType::General)
    .with_name("my-session")
    .with_owner(Did::new("did:key:alice"))
    .build();
let session_id = rhizo.create_session(session).await?;
```

### Vertex Creation
```rust
use rhizo_crypt_core::{VertexBuilder, EventType};

let vertex = VertexBuilder::new(EventType::DataCreate { schema: None })
    .with_agent(did.clone())
    .with_parent(parent_id)  // optional
    .build();
let vertex_id = rhizo.append_vertex(session_id, vertex).await?;
```

### Primal Lifecycle
```rust
use rhizo_crypt_core::PrimalLifecycle;

let mut rhizo = RhizoCrypt::new(config);
rhizo.start().await?;
// ... use rhizoCrypt ...
rhizo.stop().await?;
```

### Lock-Free Reads
```rust
// NOT async - instant reads!
let session_info = rhizo.get_session(session_id)?;
let sessions = rhizo.list_sessions();
```

### Dehydration
```rust
let merkle_root = rhizo.dehydrate(session_id).await?;
let status = rhizo.get_dehydration_status(session_id).await;
```

---

## 📊 Current Status

| Demo Section | Total | Working | Needs Update |
|--------------|-------|---------|--------------|
| 01-hello-rhizocrypt | 3 | 1 | 2 |
| 02-dag-engine | 4 | 0 | 4 |
| 03-merkle-proofs | 4 | 0 | 4 |
| 04-sessions | 4 | ? | ? |
| 05-performance | 4 | ? | ? |
| 06-advanced | 3 | ? | ? |
| 07-dehydration | 1 | ? | ? |
| 08-production | 2 | ? | ? |
| **TOTAL** | **25** | **1** | **24** |

---

## 🎯 Update Strategy

### Phase 1: Core Demos (Priority 1)
1. ✅ 01-hello-rhizocrypt/demo-first-session.sh — DONE
2. ⏳ 01-hello-rhizocrypt/demo-first-vertex.sh
3. ⏳ 01-hello-rhizocrypt/demo-query-dag.sh
4. ⏳ 02-dag-engine/* (all 4 demos)
5. ⏳ 03-merkle-proofs/* (all 4 demos)

### Phase 2: Advanced Demos (Priority 2)
6. ⏳ 04-sessions/* (all 4 demos)
7. ⏳ 07-dehydration/demo-simple-dehydration.sh
8. ⏳ 08-production/* (all 2 demos)

### Phase 3: Showcase Demos (Priority 3)
9. ⏳ 05-performance/* (all 4 demos)
10. ⏳ 06-advanced-patterns/* (all 3 demos)
11. ⏳ 06-real-world-scenarios/* (all 4 demos)

---

## 🔧 Common API Fixes

### Old → New

| Old API | New API | Note |
|---------|---------|------|
| `Session::new()` | `SessionBuilder::new().build()` | Use builder pattern |
| `SessionType::Ephemeral` | `SessionType::General` | Enum variant changed |
| `get_session().await?` | `get_session()?` | NOT async (lock-free!) |
| `list_sessions().await` | `list_sessions()` | NOT async |
| `rhizo.stop().await?` | Import `PrimalLifecycle` trait |
| `.with_payload(bytes)` | Removed (use payload refs) |

---

## 📝 Update Checklist Per Demo

For each demo:
- [ ] Update `Session` creation to `SessionBuilder`
- [ ] Update `SessionType::Ephemeral` to `SessionType::General`
- [ ] Remove `.await` from lock-free methods
- [ ] Add `use rhizo_crypt_core::PrimalLifecycle`
- [ ] Add `Did` for agent/owner
- [ ] Test with real compilation
- [ ] Verify output is correct

---

## 🎯 Success Criteria

**Demo is polished when**:
- ✅ Compiles with current rhizoCrypt APIs
- ✅ Runs successfully (exit code 0)
- ✅ Produces expected output
- ✅ Uses validated patterns (no deprecated APIs)
- ✅ Clear educational value (comments, output)

---

## 📈 Progress Tracking

```bash
# Run this to track progress
cd showcase/00-local-primal
./test-all-demos.sh
```

**Target**: 25/25 demos working (100%)

---

**Started**: December 27, 2025  
**Phase**: 1 of 3 (Core Demos)  
**Next**: Update first-vertex and query-dag demos

