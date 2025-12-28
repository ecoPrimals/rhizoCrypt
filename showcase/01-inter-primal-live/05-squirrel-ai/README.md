# 05: Squirrel AI Routing & Intelligence

**Integration with Squirrel for intelligent routing and AI-powered decision making**

## Overview

Squirrel is the AI/intelligence primal, providing intelligent routing, pattern recognition, and adaptive decision-making. `rhizoCrypt` integrates with Squirrel to enable AI-guided workflows, smart content routing, and emergent collaboration patterns.

## Architecture

```
┌─────────────────┐         ┌─────────────────┐
│   rhizoCrypt    │         │    Squirrel     │
│   (DAG State)   │         │   (AI/Intel)    │
├─────────────────┤         ├─────────────────┤
│ • Session DAG   │  query  │ • Pattern match │
│ • Event history │────────>│ • Route suggest │
│ • Agent actions │         │ • Smart routing │
│ • Context       │         │ • Learning      │
└─────────────────┘         └─────────────────┘
```

## Demos

### 1. Intelligent Routing (`demo-intelligent-routing.sh`)
**Concept:** AI-powered content routing based on DAG patterns.

- Session context analysis
- Agent capability matching
- Dynamic route selection
- Performance optimization

**Run:**
```bash
./demo-intelligent-routing.sh
```

### 2. Pattern Recognition (`demo-pattern-recognition.sh`)
**Concept:** Detect workflow patterns in DAG history.

- Common workflows identified
- Bottleneck detection
- Collaboration patterns
- Anomaly detection

**Run:**
```bash
./demo-pattern-recognition.sh
```

### 3. Adaptive Workflows (`demo-adaptive-workflows.sh`)
**Concept:** AI adapts workflows based on learned patterns.

- Historical pattern analysis
- Workflow optimization suggestions
- Auto-routing based on context
- Continuous learning

**Run:**
```bash
./demo-adaptive-workflows.sh
```

## Key Patterns

### Routing Request
```rust
// Ask Squirrel for optimal routing
let routing_decision = squirrel_client.route_request(
    RoutingContext {
        session_id: session.id(),
        agent: did!("did:key:user"),
        content_type: "document-edit",
        dag_frontier: session.frontier().await?,
        history: session.recent_vertices(100).await?,
    }
).await?;

// Apply AI recommendation
match routing_decision.recommendation {
    Route::BearDog => sign_vertex_with_beardog().await?,
    Route::NestGate => store_payload_in_nestgate().await?,
    Route::ToadStool => execute_compute().await?,
}
```

### Pattern Detection
```rust
// Detect patterns in DAG
let patterns = squirrel_client.analyze_patterns(
    session.all_vertices().await?
).await?;

for pattern in patterns {
    println!("Pattern: {} (confidence: {})", 
        pattern.name, pattern.confidence);
    println!("Suggestion: {}", pattern.optimization);
}
```

### Adaptive Learning
```rust
// Squirrel learns from workflow outcomes
let outcome = WorkflowOutcome {
    session_id: session.id(),
    success: true,
    duration_ms: 1234,
    agent_satisfaction: 0.95,
};

squirrel_client.record_outcome(outcome).await?;

// Future workflows benefit from learning
```

## Benefits

| Aspect | Benefit |
|--------|---------|
| **Efficiency** | Optimal routing reduces latency |
| **Scalability** | AI handles complex routing logic |
| **Learning** | Improves over time |
| **Flexibility** | Adapts to changing patterns |
| **Intelligence** | Emergent collaboration |

## Real-World Use Cases

1. **Smart Document Routing**
   - Route edits to appropriate primals
   - Detect collaboration patterns
   - Optimize for latency/cost

2. **Workflow Optimization**
   - Identify common patterns
   - Suggest improvements
   - Auto-route based on context

3. **Anomaly Detection**
   - Unusual access patterns
   - Security threat detection
   - Performance issues

4. **Multi-Agent Coordination**
   - Intelligent task distribution
   - Load balancing
   - Capability matching

5. **Adaptive Systems**
   - Self-optimizing workflows
   - Continuous improvement
   - Emergent intelligence

## Technical Details

### Capability-Based Discovery
```rust
// No hardcoded Squirrel endpoints
let ai_client = CapabilityRegistry::discover("AIProvider")
    .with_capability("routing")
    .with_capability("pattern-recognition")
    .await?;
```

### Context Sharing
```rust
// Share DAG context with Squirrel
let context = AIContext {
    session_dag: session.export_dag().await?,
    agent_history: session.agent_actions(agent_did).await?,
    recent_vertices: session.frontier_with_depth(10).await?,
    metadata: session.metadata().await?,
};

let decision = ai_client.make_decision(context).await?;
```

### Privacy Preservation
```rust
// Squirrel only sees what it needs
let anonymized_context = session.export_dag()
    .anonymize_agents()  // Remove agent DIDs
    .truncate_payloads() // Strip sensitive data
    .await?;

ai_client.analyze(anonymized_context).await?;
```

## Integration Philosophy

### "Intelligence Where It Matters"
- rhizoCrypt: Fast ephemeral DAG
- Squirrel: Smart routing and patterns
- Humans: Final decisions

### No Lock-In
- Pure capability-based discovery
- Works with any AI provider
- Graceful fallback to heuristics

### Consent-Based
- Agents opt-in to AI routing
- Privacy-preserving by default
- Transparent recommendations

## Provenance

```
Session (Collaborative Editing)
├─ Edit 1 (Alice)
├─ Routing Decision (Squirrel AI)
│  └─ Recommendation: Store in NestGate
├─ Storage (NestGate)
└─ Edit 2 (Bob)
   └─ Squirrel learned: Co-editing pattern detected

Merkle Root: Includes AI routing decisions in provenance
```

## Next Steps

Explore complete workflows with AI intelligence:
```bash
cd ../05-complete-workflows
./demo-ai-document-collaboration.sh
```

---

**No mocks. Real Squirrel binary. Intelligent, adaptive workflows.**
