# 🧪 Test Coverage Expansion Plan
**Target:** 79% → 90%+ Coverage  
**Date:** January 9, 2026  
**Status:** Ready for Implementation

---

## 🎯 Coverage Gap Analysis

### Current State
- **Coverage:** 79.35%
- **Tests:** 374 passing (100%)
- **Gap:** 10.65% to reach 90%

### Coverage Breakdown
```
E2E Tests:       14 tests ✅
Chaos Tests:     26 tests ✅
Property Tests:   7 tests ✅
Unit Tests:     327 tests ✅
```

---

## 📊 Identified Coverage Gaps

### 1. Error Path Coverage (Est. +5%)

**Missing Coverage:**
- Network timeout scenarios
- Service unavailable errors
- Malformed response handling
- Discovery failure paths
- Commit failure recovery

**Test Areas to Add:**
```rust
// Discovery error paths
test_discover_with_no_providers()
test_discover_with_timeout()
test_discover_with_malformed_response()

// Commit error paths
test_commit_network_failure()
test_commit_invalid_response()
test_commit_service_unavailable()

// Dehydration error paths
test_dehydrate_empty_session()
test_dehydrate_without_discovery()
test_dehydrate_partial_failure()
```

### 2. Edge Case Coverage (Est. +3%)

**Missing Coverage:**
- Zero-length inputs
- Maximum size limits
- Boundary conditions
- Unicode/special characters
- Concurrent edge cases

**Test Areas to Add:**
```rust
// Boundary conditions
test_session_with_zero_timeout()
test_vertex_with_empty_metadata()
test_dag_with_max_vertices()

// Unicode/special cases
test_vertex_with_unicode_content()
test_session_with_special_characters()

// Concurrent boundaries
test_max_concurrent_sessions()
test_concurrent_append_race_conditions()
```

### 3. Recovery Scenarios (Est. +3%)

**Missing Coverage:**
- State recovery after crash
- Partial commit recovery
- Session cleanup
- Resource cleanup

**Test Areas to Add:**
```rust
// Recovery paths
test_recovery_after_commit_failure()
test_recovery_from_invalid_state()
test_cleanup_after_session_discard()

// Resource management
test_memory_cleanup_after_large_dag()
test_handle_cleanup_on_error()
```

---

## 🔧 Implementation Strategy

### Phase 1: Low-Hanging Fruit (79% → 85%)
**Timeline:** 2-3 days  
**Effort:** Medium

**Actions:**
1. Add error injection to existing test harness
2. Create negative test cases for all public APIs
3. Test all error paths in async functions
4. Cover remaining branches in match statements

**Example Template:**
```rust
#[tokio::test]
async fn test_api_with_invalid_input() {
    let harness = TestHarness::new().await;
    
    // Test with invalid input
    let result = harness.rhizo.method_name(invalid_input).await;
    
    // Verify error handling
    assert!(matches!(result, Err(RhizoCryptError::InvalidInput(_))));
}
```

### Phase 2: Edge Cases (85% → 90%)
**Timeline:** 3-4 days  
**Effort:** High

**Actions:**
1. Test boundary conditions for all limits
2. Add stress tests for concurrent operations
3. Test with unusual but valid inputs
4. Cover error recovery paths

**Example Template:**
```rust
#[tokio::test]
async fn test_concurrent_stress() {
    let harness = TestHarness::new().await;
    let mut handles = vec![];
    
    for _ in 0..1000 {
        let h = harness.clone();
        handles.push(tokio::spawn(async move {
            h.rhizo.operation().await
        }));
    }
    
    // All should succeed or fail gracefully
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok() || is_expected_error(result));
    }
}
```

### Phase 3: Stretch Goal (90% → 95%)
**Timeline:** 1-2 weeks  
**Effort:** Very High

**Actions:**
1. Exhaustive state machine testing
2. Byzantine failure scenarios
3. Long-running stability tests
4. Memory leak detection

---

## 📁 Test File Organization

### New Test Files to Create

1. **`tests/error_paths/mod.rs`**
   - Discovery errors
   - Network errors
   - Validation errors
   - State errors

2. **`tests/edge_cases/mod.rs`**
   - Boundary conditions
   - Concurrent scenarios
   - Large-scale tests
   - Unicode/special chars

3. **`tests/recovery/mod.rs`**
   - Failure recovery
   - State recovery
   - Resource cleanup
   - Graceful degradation

### Existing Files to Expand

1. **`tests/e2e/session_lifecycle.rs`**
   - Add error path tests
   - Add timeout scenarios
   - Add cleanup verification

2. **`tests/chaos/failure_injection.rs`**
   - Add more failure scenarios
   - Add partial failure tests
   - Add recovery verification

---

## 🎯 Specific Tests to Implement

### Priority 1: Core API Error Paths

```rust
// Session API errors
✅ test_create_session_invalid_config()
✅ test_append_vertex_to_nonexistent_session()
✅ test_discard_nonexistent_session()
✅ test_compute_merkle_root_invalid_session()

// Dehydration errors
✅ test_dehydrate_empty_session()
✅ test_dehydrate_without_storage_provider()
✅ test_dehydrate_with_network_failure()
✅ test_dehydrate_with_invalid_commitment()

// Discovery errors
✅ test_discover_with_no_services()
✅ test_discover_with_timeout()
✅ test_discover_with_invalid_response()
```

### Priority 2: Boundary Conditions

```rust
// Size boundaries
✅ test_session_with_zero_vertices()
✅ test_session_with_max_vertices()
✅ test_vertex_with_empty_metadata()
✅ test_vertex_with_max_metadata()

// Time boundaries
✅ test_session_with_zero_timeout()
✅ test_session_with_max_timeout()
✅ test_expired_session_behavior()

// Concurrent boundaries
✅ test_max_concurrent_sessions()
✅ test_concurrent_vertex_append()
✅ test_concurrent_dehydration()
```

### Priority 3: Recovery Scenarios

```rust
// Failure recovery
✅ test_recover_from_commit_failure()
✅ test_recover_from_partial_dehydration()
✅ test_retry_on_transient_error()

// Resource cleanup
✅ test_cleanup_after_session_discard()
✅ test_cleanup_after_failed_dehydration()
✅ test_memory_release_on_error()
```

---

## 📊 Coverage Tracking

### Measurement Commands

```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace --html

# Generate detailed line coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# View coverage summary
cargo llvm-cov --all-features --workspace --summary-only

# Coverage by file
cargo llvm-cov --all-features --workspace --json | jq '.data[0].files[] | {file:.filename, coverage:.summary.lines.percent}'
```

### Target Metrics

| Phase | Target | Tests Added | Timeline |
|-------|--------|-------------|----------|
| Current | 79.35% | 374 | Baseline |
| Phase 1 | 85% | +30-40 | 2-3 days |
| Phase 2 | 90% | +40-50 | 5-7 days |
| Phase 3 | 95% | +50-100 | 2-3 weeks |

---

## 🔍 Files Needing Additional Coverage

### High Priority (Core Functionality)

1. **`rhizocrypt.rs`** (756 lines)
   - Current est: ~75%
   - Target: 90%+
   - Focus: Error paths in async methods

2. **`dehydration_impl.rs`** 
   - Current est: ~70%
   - Target: 90%+
   - Focus: Failure scenarios, edge cases

3. **`discovery.rs`** (762 lines)
   - Current est: ~80%
   - Target: 95%+
   - Focus: Network error handling

### Medium Priority (Client Code)

4. **`clients/loamspine_http.rs`** (497 lines)
   - Current est: ~60%
   - Target: 85%+
   - Focus: HTTP error responses

5. **`clients/capabilities/*.rs`**
   - Current est: ~70%
   - Target: 90%+
   - Focus: Discovery failure paths

### Lower Priority (Supporting Code)

6. **`safe_env.rs`** (761 lines)
   - Current est: ~85%
   - Target: 95%+
   - Focus: Edge cases in environment parsing

7. **`types_ecosystem/*.rs`**
   - Current est: ~75%
   - Target: 90%+
   - Focus: Serialization edge cases

---

## 🏗️ Testing Infrastructure Improvements

### 1. Enhanced Test Harness

```rust
// Add to tests/common/harness.rs
pub struct TestHarness {
    pub rhizo: Arc<RhizoCrypt>,
    pub mock_discovery: Arc<MockDiscoveryRegistry>,
    pub error_injector: Arc<ErrorInjector>,
}

impl TestHarness {
    pub async fn with_error_injection() -> Self {
        // Create harness with error injection capabilities
    }
    
    pub async fn with_slow_network() -> Self {
        // Simulate network latency
    }
    
    pub async fn with_failing_storage() -> Self {
        // Simulate storage failures
    }
}
```

### 2. Error Injection Framework

```rust
pub struct ErrorInjector {
    failure_rate: f64,
    error_type: ErrorType,
}

impl ErrorInjector {
    pub fn should_fail(&self) -> bool {
        rand::random::<f64>() < self.failure_rate
    }
    
    pub fn inject_error<T>(&self, result: Result<T>) -> Result<T> {
        if self.should_fail() {
            Err(self.error_type.into())
        } else {
            result
        }
    }
}
```

### 3. Property-Based Testing Expansion

```rust
// Expand property_tests.rs with more generators
proptest! {
    #[test]
    fn prop_session_always_has_creator(
        session_type in any_session_type(),
        creator in any_did(),
    ) {
        let config = SessionConfig::builder()
            .session_type(session_type)
            .creator(creator)
            .build()?;
        
        prop_assert_eq!(config.creator, creator);
    }
}
```

---

## ✅ Success Criteria

### Coverage Metrics
- [ ] Overall coverage ≥ 90%
- [ ] Core modules coverage ≥ 95%
- [ ] Client modules coverage ≥ 85%
- [ ] All public APIs have error path tests

### Quality Metrics
- [ ] Zero test flakiness
- [ ] All tests pass consistently
- [ ] No increase in test runtime >20%
- [ ] Clear test documentation

### Documentation
- [ ] All new tests have clear descriptions
- [ ] Coverage gaps documented
- [ ] Rationale for untested code documented

---

## 📝 Notes for Implementation

### Best Practices

1. **Use existing test harness** - Don't duplicate infrastructure
2. **Follow naming conventions** - `test_<module>_<scenario>_<expected>`
3. **Group related tests** - Use modules for organization
4. **Document complex scenarios** - Add comments explaining edge cases
5. **Keep tests focused** - One scenario per test

### Common Pitfalls to Avoid

1. ❌ Testing implementation details
2. ❌ Flaky async tests
3. ❌ Tests that depend on external state
4. ❌ Tests that take too long (>5s)
5. ❌ Tests without clear assertions

### Code Review Checklist

- [ ] Test has clear descriptive name
- [ ] Test follows AAA pattern (Arrange, Act, Assert)
- [ ] Test is independent (no shared state)
- [ ] Test is deterministic (no random failures)
- [ ] Test has meaningful assertions
- [ ] Test failure message is clear

---

## 🚀 Deployment Plan

### Week 1: Phase 1 (79% → 85%)
- Day 1-2: Implement error path tests
- Day 3: Run coverage, identify gaps
- Day 4: Fill remaining gaps
- Day 5: Review and refine

### Week 2: Phase 2 (85% → 90%)
- Day 1-2: Implement edge case tests
- Day 3-4: Implement recovery tests
- Day 5: Run full coverage analysis

### Week 3+: Phase 3 (90% → 95%)
- Stretch goal: Exhaustive testing
- Only if time permits and ROI justified

---

## 📊 Progress Tracking

| Date | Coverage | Tests | Delta | Notes |
|------|----------|-------|-------|-------|
| Jan 9 | 79.35% | 374 | Baseline | Initial audit |
| TBD | 85% | ~410 | +5.65% | Phase 1 complete |
| TBD | 90% | ~460 | +10.65% | Phase 2 complete |
| TBD | 95% | ~520 | +15.65% | Phase 3 (stretch) |

---

**Status:** Plan Complete, Ready for Implementation  
**Next Action:** Begin Phase 1 error path test implementation  
**Estimated Completion:** 2-3 weeks for 90% target

---

*Testing is not about perfection, it's about confidence.* ✅
