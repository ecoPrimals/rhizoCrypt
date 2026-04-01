# Lib.rs Refactoring Plan

**Current Status**: 1,102 lines (102 lines over limit)

## Analysis

### Method Count by Category
- **Session Management**: 4 methods (~140 lines)
  - `create_session`, `get_session`, `list_sessions`, `discard_session`
- **DAG Operations**: 8 methods (~170 lines)
  - `append_vertex`, `get_vertex`, `get_all_vertices`, `query_vertices`
  - `compute_merkle_root`, `generate_merkle_proof`
  - `session_count`, `total_vertex_count`
- **Slice Management**: 4 methods (~75 lines)
  - `checkout_slice`, `get_slice`, `list_slices`, `resolve_slice`
- **Dehydration**: 4 methods (~235 lines) ⭐ **LARGEST SECTION**
  - `dehydrate`, `get_dehydration_status`
  - `collect_attestations`, `commit_to_permanent_storage`

### Conservative Extraction Strategy

**Goal**: Get under 1000 lines with minimal risk

**Phase 1**: Extract dehydration logic (~235 lines)
- Create `rhizocrypt_core.rs` module
- Move 4 dehydration methods
- Keep all other logic in `lib.rs`
- **Result**: ~867 lines (133 under limit) ✅

**If needed - Phase 2**: Extract helpers (~50-100 lines)
- Move internal helper functions
- **Result**: ~770-820 lines

## Extraction Approach

### Conservative Pattern (Learned from previous attempt)
1. Create module file with methods as free functions
2. Add `impl RhizoCrypt` block in module with delegation
3. Import module in lib.rs
4. Keep all struct definitions in lib.rs
5. Make fields pub(crate) as needed
6. Test after each extraction

### Why Dehydration First?
- ✅ Largest section (235 lines = 21% of file)
- ✅ Most self-contained (clear boundaries)
- ✅ Already has helper methods (`collect_attestations`, `commit_to_permanent_storage`)
- ✅ Uses well-defined types (`DehydrationStatus`, `DehydrationSummary`, etc.)
- ✅ Minimal cross-cutting concerns

### Risk Assessment
- **Low Risk**: Dehydration methods mostly self-contained
- **Medium Risk**: May need to expose some DashMap fields as pub(crate)
- **Low Risk**: Types already exist in `dehydration` module

## Execution Plan

1. **Create `crates/rhizo-crypt-core/src/rhizocrypt_core.rs`**
2. **Move dehydration methods** (lines 717-950)
3. **Update lib.rs** - remove moved methods, add `mod rhizocrypt_core;`
4. **Make fields pub(crate) if needed**
5. **Run tests**
6. **Verify line count < 1000**

## Expected Outcome

```
Before: 1,102 lines
After:  ~867 lines (235 lines extracted)
Status: ✅ UNDER 1000 LINE LIMIT
```

---

*Generated: December 27, 2025*

