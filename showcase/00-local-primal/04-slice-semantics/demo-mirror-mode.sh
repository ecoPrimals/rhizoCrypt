#!/usr/bin/env bash
# Demo: Slice Mirror Mode - Synchronized Copy Across Locations
# Time: 3 minutes
# Demonstrates: Real-time synchronization between ephemeral and permanent

set -euo pipefail

echo ""
echo "🔐 rhizoCrypt - Slice Semantics: Mirror Mode"
echo "============================================="
echo ""
echo "Goal: Keep ephemeral DAG synchronized with permanent storage"
echo ""

# Mirror mode creates a live, synchronized copy. Changes in one location
# are reflected in the other. Perfect for real-time collaboration or
# keeping a working copy in sync with production.

echo "📚 Use Case: Real-Time Collaborative Editing"
echo ""
echo "Scenario: A team of data scientists are collaborating on a"
echo "machine learning experiment. They need to:"
echo "  1. Work on the same dataset simultaneously"
echo "  2. See each other's changes in real-time"
echo "  3. Maintain sync with permanent storage"
echo "  4. Avoid conflicts and data loss"
echo ""

sleep 2

echo "Step 1: Original dataset in permanent storage"
echo "----------------------------------------------"
echo ""
echo "Team's LoamSpine:"
echo "  → Commit ID: loam-commit-ml-experiment-baseline"
echo "  → Data: Training dataset (100k samples)"
echo "  → Owner: ML Team"
echo "  → Status: Active experiment"
echo ""

sleep 2

echo "Step 2: Mirror checkout for collaboration"
echo "------------------------------------------"
echo ""
echo "Request to rhizoCrypt:"
cat << 'YAML'
slice_request:
  mode: Mirror
  source: loam-commit-ml-experiment-baseline
  sync_direction: bidirectional
  collaborators:
    - did:example:data-scientist-alice
    - did:example:data-scientist-bob
    - did:example:ml-engineer-charlie
  sync_frequency: real_time
  conflict_resolution: last_write_wins
  reason: "Team collaboration on ML experiment"
YAML
echo ""

sleep 2

echo "Step 3: rhizoCrypt creates mirrored slice"
echo "------------------------------------------"
echo ""
echo "✅ Mirror established"
echo "  → Mirror ID: mirror-xyz789"
echo "  → Session ID: session-mirror-001"
echo "  → Source: LoamSpine (loam-commit-ml-experiment-baseline)"
echo "  → Mirror: rhizoCrypt ephemeral DAG"
echo "  → Sync: Bidirectional, real-time"
echo "  → Collaborators: 3 (Alice, Bob, Charlie)"
echo ""

sleep 2

echo "🔄 Synchronization active:"
echo "  → Changes in LoamSpine → pushed to rhizoCrypt"
echo "  → Changes in rhizoCrypt → pushed to LoamSpine"
echo "  → All collaborators see updates instantly"
echo ""

sleep 2

echo "Step 4: Alice adds a feature"
echo "----------------------------"
echo ""
echo "Alice adds a new feature to the dataset:"
echo ""
echo "Vertex 1:"
echo "  → Event: FeatureEngineering"
echo "  → Agent: did:example:data-scientist-alice"
echo "  → Data: {feature: 'normalized_age', method: 'min-max'}"
echo "  → Timestamp: 2025-12-26T10:00:00Z"
echo ""

sleep 1

echo "🔄 Syncing to LoamSpine..."
echo "  → Change detected in rhizoCrypt"
echo "  → Pushing to permanent storage..."
echo "  → ✅ Synced to LoamSpine in 50ms"
echo ""

sleep 1

echo "📢 Notifying collaborators..."
echo "  → Bob sees: 'Alice added normalized_age feature'"
echo "  → Charlie sees: 'Alice added normalized_age feature'"
echo ""

sleep 2

echo "Step 5: Bob makes simultaneous change"
echo "--------------------------------------"
echo ""
echo "Bob adds a different feature at the same time:"
echo ""
echo "Vertex 2:"
echo "  → Event: FeatureEngineering"
echo "  → Agent: did:example:data-scientist-bob"
echo "  → Data: {feature: 'income_category', method: 'binning'}"
echo "  → Timestamp: 2025-12-26T10:00:15Z (15 seconds later)"
echo ""

sleep 1

echo "🔄 Syncing to LoamSpine..."
echo "  → Change detected in rhizoCrypt"
echo "  → No conflict (different features)"
echo "  → ✅ Synced to LoamSpine in 45ms"
echo ""

sleep 1

echo "✅ Both changes now in permanent storage"
echo "✅ All collaborators see both features"
echo ""

sleep 2

echo "Step 6: Conflict detection and resolution"
echo "------------------------------------------"
echo ""
echo "Alice and Charlie both try to normalize the same feature:"
echo ""
echo "Alice (at 10:05:00):"
echo "Vertex 3a:"
echo "  → Event: FeatureUpdate"
echo "  → Feature: income_category"
echo "  → Change: 'Convert to log scale'"
echo ""

sleep 1

echo "Charlie (at 10:05:02 - 2 seconds later):"
echo "Vertex 3b:"
echo "  → Event: FeatureUpdate"
echo "  → Feature: income_category"
echo "  → Change: 'Convert to percentile ranks'"
echo ""

sleep 1

echo "⚠️  Conflict detected!"
echo "  → Same feature modified by two people"
echo "  → Conflict resolution: last_write_wins"
echo "  → Winner: Charlie (2 seconds later)"
echo ""

sleep 1

echo "🔄 Resolving conflict..."
echo "Vertex 4:"
echo "  → Event: ConflictResolved"
echo "  → Conflict: income_category normalization"
echo "  → Winner: Charlie (percentile ranks)"
echo "  → Loser: Alice (log scale)"
echo "  → Loser change: Preserved in DAG history"
echo ""

sleep 2

echo "📢 Notification to Alice:"
echo "  'Your change to income_category was overwritten by Charlie.'"
echo "  'Would you like to: [Undo Charlie's change] [Keep both] [Discuss]'"
echo ""

sleep 2

echo "✅ Conflict resolved, sync continues"
echo "✅ Alice's change preserved in history for rollback if needed"
echo ""

sleep 2

echo "Step 7: Real-time collaboration continues"
echo "------------------------------------------"
echo ""
echo "Team continues working in real-time:"
echo ""
echo "10:10 - Alice: Adds validation rules"
echo "10:12 - Bob: Removes outliers"
echo "10:15 - Charlie: Splits dataset for cross-validation"
echo "10:18 - All: Review changes together"
echo ""

sleep 1

echo "All changes synced in real-time:"
echo "  → Total vertices: 15"
echo "  → Conflicts: 1 (resolved)"
echo "  → Sync latency: avg 48ms"
echo "  → All collaborators in sync"
echo ""

sleep 2

echo "Step 8: Stopping the mirror"
echo "---------------------------"
echo ""
echo "Team completes initial data prep, stops mirroring:"
echo ""
echo "Vertex 16:"
echo "  → Event: MirrorStopped"
echo "  → Reason: 'Data prep complete, ready for training'"
echo "  → Final sync: Initiated"
echo ""

sleep 1

echo "🔄 Final synchronization..."
echo "  → Pushing all remaining changes to LoamSpine"
echo "  → Verifying data integrity"
echo "  → Computing final Merkle root"
echo ""

sleep 1

echo "✅ Mirror stopped successfully"
echo "  → Final state committed to: loam-commit-ml-experiment-prepped"
echo "  → All 15 vertices preserved"
echo "  → Sync history: Complete audit trail"
echo "  → Collaborators can continue independently"
echo ""

sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Key Insights: Mirror Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Real-Time Sync:"
echo "   Changes propagate instantly (typically <100ms)."
echo ""
echo "✅ Bidirectional:"
echo "   Updates flow both directions: ephemeral ↔ permanent."
echo ""
echo "✅ Multi-Collaborator:"
echo "   Multiple agents can work simultaneously."
echo ""
echo "✅ Conflict Resolution:"
echo "   Automatic conflict handling (last-write-wins, merge, etc.)."
echo ""
echo "✅ Audit Trail:"
echo "   Every change, conflict, resolution tracked in DAG."
echo ""
echo "✅ Best For:"
echo "   • Real-time collaboration (team editing)"
echo "   • Live experiments (ongoing analysis)"
echo "   • Synchronized backups (keep working + permanent in sync)"
echo "   • Multi-location work (laptop + cloud)"
echo "   • Continuous integration (dev → prod sync)"
echo ""
echo "❌ NOT For:"
echo "   • One-time access (use Loan or Copy)"
echo "   • No need for sync (use Copy mode)"
echo "   • High-latency scenarios (sync overhead)"
echo "   • Conflicting work patterns (use separate sessions)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Real-World Examples:"
echo ""
echo "1. ML Team Collaboration:"
echo "   - 3 data scientists work on same experiment"
echo "   - See each other's changes in real-time"
echo "   - Automatic conflict resolution"
echo "   - Instant sync to permanent storage"
echo ""
echo "2. Continuous Integration:"
echo "   - Dev environment mirrors production data"
echo "   - Changes in dev → automatically tested"
echo "   - Approved changes → sync to production"
echo "   - Full audit trail of all changes"
echo ""
echo "3. Multi-Device Work:"
echo "   - Laptop + cloud server stay in sync"
echo "   - Work offline, sync when reconnected"
echo "   - Conflicts resolved automatically"
echo "   - Never lose work"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 Comparison:"
echo "   • Copy: Static, no sync"
echo "   • Loan: Temporary, no sync"
echo "   • Mirror: Real-time bidirectional sync ✅"
echo "   • Provenance: Read-only, no sync"
echo ""
echo "✅ Demo complete! Mirror mode enables real-time collaboration."
echo ""

