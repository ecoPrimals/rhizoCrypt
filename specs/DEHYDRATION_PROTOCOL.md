# RhizoCrypt — Dehydration Protocol Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

**Dehydration** is the process of committing a RhizoCrypt DAG session to permanent storage in LoamSpine. The name reflects the biological metaphor: the ephemeral, water-rich rhizome layer dries and compresses into the permanent loam fossil record.

The protocol ensures:
- Cryptographic integrity (Merkle root of session)
- Selective preservation (only important results survive)
- Slice resolution (all checked-out state is resolved)
- Garbage collection (ephemeral data is freed)

---

## 2. The Dehydration Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                    Session Lifecycle                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐    ┌──────────┐    ┌───────────┐    ┌──────────┐ │
│  │ ACTIVE   │───▶│RESOLVING │───▶│ COMMITTED │───▶│ EXPIRED  │ │
│  │          │    │          │    │           │    │          │ │
│  │ Events   │    │ Dehydrate│    │ Loam ref  │    │ GC'd     │ │
│  │ flowing  │    │ in prog  │    │ available │    │          │ │
│  └──────────┘    └──────────┘    └───────────┘    └──────────┘ │
│       │                │                                        │
│       │                │                                        │
│       ▼                ▼                                        │
│  ┌──────────┐    ┌──────────┐                                   │
│  │ PAUSED   │    │DISCARDED │                                   │
│  │          │    │          │                                   │
│  │ Temp hold│    │ Rollback │                                   │
│  └──────────┘    └──────────┘                                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.1 State Transitions

| From | To | Trigger | Action |
|------|-----|---------|--------|
| Active | Resolving | Manual resolve / Timeout / Max vertices | Begin dehydration |
| Active | Paused | Manual pause | Stop accepting events |
| Active | Discarded | Manual discard | Skip dehydration, cleanup |
| Paused | Active | Manual resume | Resume accepting events |
| Paused | Resolving | Manual resolve | Begin dehydration |
| Resolving | Committed | Dehydration success | Store Loam reference |
| Resolving | Discarded | Dehydration failure | Cleanup |
| Committed | Expired | GC sweep | Free resources |
| Discarded | Expired | GC sweep | Free resources |

---

## 3. Dehydration Protocol Steps

### 3.1 Protocol Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Dehydration Protocol                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Step 1: Freeze Session                                          │
│  ────────────────────                                            │
│  • Set state = Resolving                                         │
│  • Reject new events                                             │
│  • Record resolution timestamp                                   │
│                                                                  │
│  Step 2: Compute Merkle Root                                     │
│  ─────────────────────────────                                   │
│  • Topological sort all vertices                                 │
│  • Build Merkle tree                                             │
│  • Compute root hash                                             │
│                                                                  │
│  Step 3: Generate Summary                                        │
│  ─────────────────────────                                       │
│  • Extract session metadata                                      │
│  • Summarize agent participation                                 │
│  • Extract key results                                           │
│  • Compute statistics                                            │
│                                                                  │
│  Step 4: Resolve Slices                                          │
│  ─────────────────────                                           │
│  • Evaluate resolution routes                                    │
│  • Execute slice commits/returns                                 │
│  • Record slice resolutions                                      │
│                                                                  │
│  Step 5: Collect Attestations                                    │
│  ───────────────────────────                                     │
│  • Request signatures from required parties                      │
│  • Wait for attestation threshold                                │
│  • Timeout if attestations not received                          │
│                                                                  │
│  Step 6: Commit to LoamSpine                                     │
│  ──────────────────────────                                      │
│  • Create SessionCommit entry                                    │
│  • Append to destination spine                                   │
│  • Receive commit reference                                      │
│                                                                  │
│  Step 7: Update Session State                                    │
│  ────────────────────────────                                    │
│  • Set state = Committed                                         │
│  • Store Loam reference                                          │
│  • Schedule garbage collection                                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Step 1: Freeze Session

```rust
/// Freeze a session for dehydration
pub async fn freeze_session(
    session_id: SessionId,
    outcome: SessionOutcome,
    session_manager: &SessionManager,
) -> Result<FreezeReceipt, RhizoCryptError> {
    let session = session_manager.get_session_mut(session_id).await?;
    
    // Validate current state
    match &session.state {
        SessionState::Active => {}
        SessionState::Paused { .. } => {}
        _ => return Err(RhizoCryptError::SessionNotActive(session_id)),
    }
    
    // Transition to Resolving
    session.state = SessionState::Resolving {
        started_at: current_timestamp_nanos(),
    };
    
    // Record final state
    let receipt = FreezeReceipt {
        session_id,
        frozen_at: current_timestamp_nanos(),
        vertex_count: session.vertex_count,
        frontier_size: session.frontier.len(),
        slice_count: session.slices.len(),
        outcome,
    };
    
    Ok(receipt)
}

/// Receipt from freezing a session
#[derive(Clone, Debug)]
pub struct FreezeReceipt {
    pub session_id: SessionId,
    pub frozen_at: u64,
    pub vertex_count: u64,
    pub frontier_size: usize,
    pub slice_count: usize,
    pub outcome: SessionOutcome,
}
```

### 3.3 Step 2: Compute Merkle Root

```rust
/// Compute Merkle root for a session
pub async fn compute_merkle_root(
    session_id: SessionId,
    dag_store: &impl DagStore,
    dag_index: &DagIndex,
) -> Result<MerkleComputation, RhizoCryptError> {
    let traversal = DagTraversal::new(dag_store, dag_index);
    
    // Get vertices in topological order
    let vertices = traversal.topological_sort(session_id).await?;
    
    if vertices.is_empty() {
        return Ok(MerkleComputation {
            root: MerkleRoot([0u8; 32]),
            vertex_count: 0,
            leaf_hashes: Vec::new(),
        });
    }
    
    // Compute leaf hashes
    let leaf_hashes: Vec<ContentHash> = vertices
        .iter()
        .map(|v| v.compute_id())
        .collect();
    
    // Build Merkle tree
    let root = MerkleRoot::compute(&vertices);
    
    Ok(MerkleComputation {
        root,
        vertex_count: vertices.len(),
        leaf_hashes,
    })
}

/// Result of Merkle computation
#[derive(Clone, Debug)]
pub struct MerkleComputation {
    pub root: MerkleRoot,
    pub vertex_count: usize,
    pub leaf_hashes: Vec<ContentHash>,
}
```

### 3.4 Step 3: Generate Summary

```rust
/// Generate dehydration summary for a session
pub async fn generate_summary(
    session: &Session,
    merkle: &MerkleComputation,
    dag_store: &impl DagStore,
    dag_index: &DagIndex,
    config: &DehydrationConfig,
) -> Result<DehydrationSummary, RhizoCryptError> {
    // Compute agent participation
    let agents = compute_agent_summary(session, dag_index).await?;
    
    // Extract key results based on session type
    let results = extract_results(session, dag_store, dag_index).await?;
    
    // Compute statistics
    let stats = compute_session_stats(session, dag_store).await?;
    
    // Generate proofs for specified vertices
    let proofs = if !config.generate_proofs_for.is_empty() {
        generate_proofs(
            &config.generate_proofs_for,
            &merkle.leaf_hashes,
            &merkle.root,
            dag_store,
        ).await?
    } else {
        Vec::new()
    };
    
    Ok(DehydrationSummary {
        // Session metadata
        session_id: session.id,
        session_type: session.session_type.clone(),
        session_name: session.name.clone(),
        created_at: session.created_at,
        resolved_at: current_timestamp_nanos(),
        outcome: SessionOutcome::Success, // Set by caller
        
        // Cryptographic summary
        merkle_root: merkle.root.clone(),
        vertex_count: merkle.vertex_count as u64,
        
        // Content summary
        agents,
        results,
        stats,
        proofs,
        
        // Slice resolutions (filled in Step 4)
        slice_resolutions: Vec::new(),
        
        // Attestations (filled in Step 5)
        attestations: Vec::new(),
    })
}

/// Dehydration summary for LoamSpine commit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DehydrationSummary {
    // === Session Metadata ===
    pub session_id: SessionId,
    pub session_type: SessionType,
    pub session_name: Option<String>,
    pub created_at: u64,
    pub resolved_at: u64,
    pub outcome: SessionOutcome,
    
    // === Cryptographic Summary ===
    pub merkle_root: MerkleRoot,
    pub vertex_count: u64,
    
    // === Content Summary ===
    pub agents: Vec<AgentSummary>,
    pub results: Vec<ResultEntry>,
    pub stats: SessionStats,
    pub proofs: Vec<MerkleProof>,
    
    // === Slice Resolutions ===
    pub slice_resolutions: Vec<SliceResolution>,
    
    // === Attestations ===
    pub attestations: Vec<Attestation>,
}

/// Summary of an agent's participation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentSummary {
    pub did: Did,
    pub first_event: u64,
    pub last_event: u64,
    pub event_count: u64,
    pub event_types: Vec<String>,
}

/// A key result extracted from the session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultEntry {
    pub result_type: String,
    pub vertex_id: VertexId,
    pub payload_ref: Option<PayloadRef>,
    pub metadata: HashMap<String, Value>,
}

/// Session statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub duration_nanos: u64,
    pub total_vertices: u64,
    pub total_payload_bytes: u64,
    pub unique_agents: u64,
    pub event_type_counts: HashMap<String, u64>,
}
```

### 3.5 Step 4: Resolve Slices

```rust
/// Resolve all slices in a session
pub async fn resolve_all_slices(
    session: &Session,
    outcome: &SessionOutcome,
    summary: &mut DehydrationSummary,
    loamspine: &impl LoamSpineClient,
    beardog: &impl BearDogClient,
) -> Result<(), RhizoCryptError> {
    for (slice_id, slice) in &session.slices {
        let resolution = resolve_single_slice(
            slice,
            outcome,
            loamspine,
            beardog,
        ).await?;
        
        summary.slice_resolutions.push(SliceResolution {
            slice_id: *slice_id,
            origin: slice.origin.clone(),
            mode: slice.mode.clone(),
            outcome: resolution,
        });
    }
    
    Ok(())
}

/// A slice resolution record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliceResolution {
    pub slice_id: SliceId,
    pub origin: SliceOrigin,
    pub mode: SliceMode,
    pub outcome: ResolutionOutcome,
}
```

### 3.6 Step 5: Collect Attestations

```rust
/// Collect required attestations
pub async fn collect_attestations(
    summary: &DehydrationSummary,
    config: &DehydrationConfig,
    beardog: &impl BearDogClient,
) -> Result<Vec<Attestation>, RhizoCryptError> {
    if config.required_attestations.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut attestations = Vec::new();
    let attestation_data = summary.to_attestation_bytes();
    
    for attester_did in &config.required_attestations {
        // Request attestation
        let request = AttestationRequest {
            subject: AttestationSubject::SessionDehydration {
                session_id: summary.session_id,
                merkle_root: summary.merkle_root.clone(),
            },
            data_hash: hash_bytes(&attestation_data),
            expires_at: current_timestamp_nanos() + 300_000_000_000, // 5 min timeout
        };
        
        let attestation = beardog
            .request_attestation(attester_did, &request)
            .await
            .map_err(|e| RhizoCryptError::BearDog(e.to_string()))?;
        
        attestations.push(attestation);
    }
    
    Ok(attestations)
}

/// An attestation from a third party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attestation {
    pub attester: Did,
    pub subject_hash: ContentHash,
    pub signature: Signature,
    pub timestamp: u64,
}
```

### 3.7 Step 6: Commit to LoamSpine

```rust
/// Commit dehydration summary to LoamSpine
pub async fn commit_to_loamspine(
    summary: &DehydrationSummary,
    destination_spine: &SpineId,
    loamspine: &impl LoamSpineClient,
    signer: &impl Signer,
) -> Result<LoamCommitRef, RhizoCryptError> {
    // Create SessionCommit entry
    let entry = LoamEntry {
        entry_type: EntryType::SessionCommit {
            session_id: summary.session_id,
            session_type: summary.session_type.clone(),
            merkle_root: summary.merkle_root.clone(),
            summary: summary.clone(),
        },
        payload: None,
        metadata: HashMap::new(),
    };
    
    // Append to spine
    let commit_ref = loamspine
        .append_entry(destination_spine, entry, signer)
        .await
        .map_err(|e| RhizoCryptError::LoamSpine(e.to_string()))?;
    
    Ok(commit_ref)
}
```

### 3.8 Step 7: Update Session State

```rust
/// Finalize session after successful commit
pub async fn finalize_session(
    session_id: SessionId,
    commit_ref: LoamCommitRef,
    session_manager: &SessionManager,
) -> Result<(), RhizoCryptError> {
    let session = session_manager.get_session_mut(session_id).await?;
    
    session.state = SessionState::Committed {
        loam_ref: commit_ref,
        committed_at: current_timestamp_nanos(),
    };
    
    // Schedule garbage collection
    session_manager.schedule_gc(session_id, GC_DELAY).await?;
    
    Ok(())
}

/// Default delay before garbage collection
const GC_DELAY: Duration = Duration::from_secs(3600); // 1 hour
```

---

## 4. Complete Dehydration Flow

```rust
/// Complete dehydration of a session
pub async fn dehydrate_session(
    session_id: SessionId,
    outcome: SessionOutcome,
    destination_spine: SpineId,
    session_manager: &SessionManager,
    dag_store: &impl DagStore,
    dag_index: &DagIndex,
    loamspine: &impl LoamSpineClient,
    beardog: &impl BearDogClient,
    signer: &impl Signer,
) -> Result<DehydrationResult, RhizoCryptError> {
    // Step 1: Freeze session
    let freeze_receipt = freeze_session(session_id, outcome.clone(), session_manager).await?;
    
    // Step 2: Compute Merkle root
    let merkle = compute_merkle_root(session_id, dag_store, dag_index).await?;
    
    // Get session for remaining steps
    let session = session_manager.get_session(session_id).await?;
    
    // Step 3: Generate summary
    let mut summary = generate_summary(
        &session,
        &merkle,
        dag_store,
        dag_index,
        &session.config.dehydration,
    ).await?;
    summary.outcome = outcome.clone();
    
    // Step 4: Resolve slices
    resolve_all_slices(&session, &outcome, &mut summary, loamspine, beardog).await?;
    
    // Step 5: Collect attestations
    let attestations = collect_attestations(
        &summary,
        &session.config.dehydration,
        beardog,
    ).await?;
    summary.attestations = attestations;
    
    // Step 6: Commit to LoamSpine
    let commit_ref = commit_to_loamspine(&summary, &destination_spine, loamspine, signer).await?;
    
    // Step 7: Finalize session
    finalize_session(session_id, commit_ref.clone(), session_manager).await?;
    
    Ok(DehydrationResult {
        session_id,
        commit_ref,
        summary,
        freeze_receipt,
    })
}

/// Result of dehydration
#[derive(Clone, Debug)]
pub struct DehydrationResult {
    pub session_id: SessionId,
    pub commit_ref: LoamCommitRef,
    pub summary: DehydrationSummary,
    pub freeze_receipt: FreezeReceipt,
}
```

---

## 5. Garbage Collection

After a session is committed or discarded, its ephemeral data must be cleaned up.

### 5.1 GC Process

```rust
/// Garbage collect an expired session
pub async fn gc_session(
    session_id: SessionId,
    session_manager: &SessionManager,
    dag_store: &impl DagStore,
    payload_store: &impl PayloadStore,
) -> Result<GcStats, RhizoCryptError> {
    let session = session_manager.get_session(session_id).await?;
    
    // Validate session is ready for GC
    match &session.state {
        SessionState::Committed { .. } | SessionState::Discarded { .. } => {}
        _ => return Err(RhizoCryptError::SessionNotActive(session_id)),
    }
    
    // Collect payload references
    let payload_refs = collect_payload_refs(&session, dag_store).await?;
    
    // Delete vertices
    let vertices_deleted = dag_store.delete_session(session_id).await?;
    
    // Delete orphaned payloads (not referenced by other sessions)
    let payloads_deleted = payload_store.gc(&payload_refs).await?;
    
    // Update session state
    let mut session = session_manager.get_session_mut(session_id).await?;
    session.state = SessionState::Expired {
        expired_at: current_timestamp_nanos(),
    };
    
    // Remove from active session list
    session_manager.remove_session(session_id).await?;
    
    Ok(GcStats {
        session_id,
        vertices_deleted,
        payloads_deleted: payloads_deleted.count,
        bytes_reclaimed: payloads_deleted.bytes,
    })
}

/// GC statistics
#[derive(Clone, Debug)]
pub struct GcStats {
    pub session_id: SessionId,
    pub vertices_deleted: u64,
    pub payloads_deleted: u64,
    pub bytes_reclaimed: u64,
}
```

### 5.2 Background GC Sweep

```rust
/// Background task for garbage collection
pub async fn gc_sweep_task(
    session_manager: Arc<SessionManager>,
    dag_store: Arc<dyn DagStore>,
    payload_store: Arc<dyn PayloadStore>,
    interval: Duration,
) {
    let mut interval_timer = tokio::time::interval(interval);
    
    loop {
        interval_timer.tick().await;
        
        // Find sessions ready for GC
        let ready_sessions = session_manager
            .list_sessions(SessionFilter::ReadyForGc)
            .await
            .unwrap_or_default();
        
        for session_summary in ready_sessions {
            match gc_session(
                session_summary.id,
                &session_manager,
                &dag_store,
                &payload_store,
            ).await {
                Ok(stats) => {
                    tracing::info!(
                        session_id = %stats.session_id,
                        vertices = stats.vertices_deleted,
                        bytes = stats.bytes_reclaimed,
                        "GC completed"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        session_id = %session_summary.id,
                        error = %e,
                        "GC failed"
                    );
                }
            }
        }
    }
}
```

---

## 6. Session Outcomes

```rust
/// Outcome of a session
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionOutcome {
    /// Session completed successfully
    Success,
    
    /// Session failed (application-level failure)
    Failure {
        reason: String,
    },
    
    /// Session was rolled back (no permanent changes)
    Rollback,
    
    /// Session timed out
    Timeout,
    
    /// Session was manually discarded
    Discarded {
        reason: String,
    },
}

impl SessionOutcome {
    /// Check if this outcome results in a LoamSpine commit
    pub fn should_commit(&self) -> bool {
        match self {
            Self::Success => true,
            Self::Failure { .. } => true, // Commit failure record
            Self::Rollback => false,
            Self::Timeout => false,
            Self::Discarded { .. } => false,
        }
    }
    
    /// Check if slices should resolve normally
    pub fn should_resolve_slices(&self) -> bool {
        match self {
            Self::Success => true,
            Self::Failure { .. } => true,
            Self::Rollback => false, // Slices return unchanged
            Self::Timeout => true,   // Slices may have timeout routes
            Self::Discarded { .. } => false,
        }
    }
}
```

---

## 7. Error Handling

### 7.1 Dehydration Errors

```rust
/// Errors during dehydration
#[derive(Debug, thiserror::Error)]
pub enum DehydrationError {
    #[error("Session not in valid state for dehydration")]
    InvalidSessionState,
    
    #[error("Merkle computation failed: {0}")]
    MerkleComputation(String),
    
    #[error("Slice resolution failed: {slice_id}")]
    SliceResolutionFailed {
        slice_id: SliceId,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Required attestation not received: {attester}")]
    AttestationTimeout {
        attester: Did,
    },
    
    #[error("LoamSpine commit failed: {0}")]
    CommitFailed(String),
    
    #[error("Dehydration aborted")]
    Aborted,
}
```

### 7.2 Partial Failure Handling

If dehydration fails partway through:

1. **Slice resolutions are atomic per-slice** — Each slice resolves independently
2. **LoamSpine commit is atomic** — Either the commit succeeds or fails entirely
3. **Session state tracks progress** — Can resume from failure point
4. **Rollback is always safe** — Slices return to origin unchanged

---

## 8. Configuration

```rust
/// Dehydration configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DehydrationConfig {
    /// Include full vertex data in summary (expensive!)
    pub include_vertices: bool,
    
    /// Include payload references in summary
    pub include_payloads: bool,
    
    /// Generate Merkle proofs for specific vertices
    pub generate_proofs_for: Vec<VertexId>,
    
    /// Required attestations before commit
    pub required_attestations: Vec<Did>,
    
    /// Attestation timeout
    pub attestation_timeout: Duration,
    
    /// Destination spine for commit
    pub destination_spine: Option<SpineId>,
    
    /// Auto-GC delay after commit
    pub gc_delay: Duration,
    
    /// Result extraction rules
    pub result_extractors: Vec<ResultExtractor>,
}

/// Rule for extracting results from a session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultExtractor {
    /// Event types to extract
    pub event_types: Vec<String>,
    
    /// Maximum results to extract
    pub max_results: Option<usize>,
    
    /// Include payload in result
    pub include_payload: bool,
    
    /// Metadata fields to include
    pub metadata_fields: Vec<String>,
}
```

---

## 9. Verification

After dehydration, the session can be verified:

```rust
/// Verify a committed session
pub async fn verify_committed_session(
    commit_ref: &LoamCommitRef,
    loamspine: &impl LoamSpineClient,
) -> Result<VerificationResult, RhizoCryptError> {
    // Get the commit entry
    let entry = loamspine
        .get_entry(&commit_ref.spine_id, &commit_ref.entry_hash)
        .await
        .map_err(|e| RhizoCryptError::LoamSpine(e.to_string()))?
        .ok_or(RhizoCryptError::LoamSpine("Commit not found".into()))?;
    
    // Extract summary
    let summary = match &entry.entry_type {
        EntryType::SessionCommit { summary, .. } => summary,
        _ => return Err(RhizoCryptError::LoamSpine("Not a session commit".into())),
    };
    
    // Verify attestations
    let mut attestation_results = Vec::new();
    for attestation in &summary.attestations {
        // Verify each attestation signature
        let valid = verify_attestation(attestation, &summary.merkle_root).await?;
        attestation_results.push((attestation.attester.clone(), valid));
    }
    
    Ok(VerificationResult {
        session_id: summary.session_id,
        merkle_root: summary.merkle_root.clone(),
        vertex_count: summary.vertex_count,
        attestations: attestation_results,
        slice_resolutions: summary.slice_resolutions.len(),
        valid: true,
    })
}

/// Verification result
#[derive(Clone, Debug)]
pub struct VerificationResult {
    pub session_id: SessionId,
    pub merkle_root: MerkleRoot,
    pub vertex_count: u64,
    pub attestations: Vec<(Did, bool)>,
    pub slice_resolutions: usize,
    pub valid: bool,
}
```

---

## 10. References

- [RHIZOCRYPT_SPECIFICATION.md](./RHIZOCRYPT_SPECIFICATION.md) — Full specification
- [DATA_MODEL.md](./DATA_MODEL.md) — Core data structures
- [SLICE_SEMANTICS.md](./SLICE_SEMANTICS.md) — Slice resolution
- [LoamSpine Specification](../../loamSpine/specs/) — Permanent layer

---

*RhizoCrypt: The memory that knows when to forget.*

