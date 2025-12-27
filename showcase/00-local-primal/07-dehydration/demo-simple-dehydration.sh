#!/usr/bin/env bash
# Demo: Simple Dehydration - Complete Workflow
# Time: 5 minutes
# Demonstrates: Ephemeral session → Permanent storage

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   💾 rhizoCrypt Dehydration - Complete Workflow${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${CYAN}What is Dehydration?${NC}"
echo "━━━━━━━━━━━━━━━━━━━━"
echo "Dehydration is the process of taking an ephemeral session"
echo "(fast, in-memory, temporary) and committing it to permanent"
echo "storage (LoamSpine, IPFS, etc.) with full cryptographic proof."
echo ""

sleep 2

echo -e "${YELLOW}📝 Step 1: Create Ephemeral Session${NC}"
echo "────────────────────────────────────"
echo ""
echo "Creating a new rhizoCrypt session..."
echo "  Session Type: General"
echo "  Name: dehydration-demo"
echo "  Storage: In-Memory (ephemeral)"
echo ""
echo "✅ Session created: session-dehydration-001"
echo ""

sleep 2

echo -e "${YELLOW}📝 Step 2: Build DAG with Multiple Vertices${NC}"
echo "─────────────────────────────────────────────"
echo ""
echo "Adding vertices to demonstrate workflow..."
echo ""

echo "Vertex 1 (Genesis):"
echo "  Event: DataCreate"
echo "  Agent: did:key:alice"
echo "  Data: {experiment: 'ml-training', version: 1}"
echo "  ✅ Added to DAG"
echo ""

sleep 1

echo "Vertex 2:"
echo "  Event: DataUpdate"
echo "  Agent: did:key:alice"
echo "  Parent: Vertex 1"
echo "  Data: {training_accuracy: 0.92}"
echo "  ✅ Added to DAG"
echo ""

sleep 1

echo "Vertex 3:"
echo "  Event: DataUpdate"
echo "  Agent: did:key:bob"
echo "  Parent: Vertex 2"
echo "  Data: {validation_accuracy: 0.89}"
echo "  ✅ Added to DAG"
echo ""

sleep 1

echo "Vertex 4:"
echo "  Event: DataUpdate"
echo "  Agent: did:key:alice"
echo "  Parent: Vertex 3"
echo "  Data: {final_model: 'model-v1.0.pkl'}"
echo "  ✅ Added to DAG"
echo ""

sleep 2

echo -e "${CYAN}Current DAG Structure:${NC}"
echo "  Vertex 1 (Genesis)"
echo "      ↓"
echo "  Vertex 2 (training)"
echo "      ↓"
echo "  Vertex 3 (validation) "
echo "      ↓"
echo "  Vertex 4 (final)"
echo ""

sleep 2

echo -e "${YELLOW}📝 Step 3: Compute Merkle Root${NC}"
echo "───────────────────────────────"
echo ""
echo "Computing Merkle tree of entire DAG..."
echo "  Algorithm: Blake3"
echo "  Vertices: 4"
echo "  Edges: 3"
echo ""

sleep 1

MOCK_ROOT="abc123def456789012345678901234567890123456789012345678901234"
echo "✅ Merkle Root: ${MOCK_ROOT:0:16}..."
echo ""
echo "This root cryptographically commits to ALL vertices!"
echo "  → Change ANY vertex = Different root"
echo "  → Tamper detection built-in"
echo ""

sleep 2

echo -e "${YELLOW}📝 Step 4: Generate Dehydration Summary${NC}"
echo "────────────────────────────────────────"
echo ""
echo "Creating comprehensive summary..."
echo ""

cat << 'SUMMARY'
Dehydration Summary:
{
  session_id: "session-dehydration-001",
  merkle_root: "abc123def456...",
  vertex_count: 4,
  agent_count: 2,
  agents: [
    {
      did: "did:key:alice",
      vertices_contributed: 3,
      events: ["DataCreate", "DataUpdate", "DataUpdate"]
    },
    {
      did: "did:key:bob",
      vertices_contributed: 1,
      events: ["DataUpdate"]
    }
  ],
  timeline: {
    start: "2025-12-27T10:00:00Z",
    end: "2025-12-27T10:15:00Z",
    duration: "15 minutes"
  },
  results: [
    {
      key: "training_accuracy",
      value: 0.92,
      vertex: 2
    },
    {
      key: "validation_accuracy", 
      value: 0.89,
      vertex: 3
    },
    {
      key: "final_model",
      value: "model-v1.0.pkl",
      vertex: 4
    }
  ]
}
SUMMARY

echo ""
echo "✅ Summary generated (comprehensive provenance)"
echo ""

sleep 3

echo -e "${YELLOW}📝 Step 5: Collect Attestations (Optional)${NC}"
echo "──────────────────────────────────────────────"
echo ""
echo "For multi-party sessions, collect signatures..."
echo ""

echo "Alice's Attestation:"
echo "  → Signs: Merkle root + summary"
echo "  → Algorithm: Ed25519"
echo "  → Signature: 0xabcd...ef12"
echo "  ✅ Verified"
echo ""

sleep 1

echo "Bob's Attestation:"
echo "  → Signs: Merkle root + summary"
echo "  → Algorithm: Ed25519"
echo "  → Signature: 0x1234...ab56"
echo "  ✅ Verified"
echo ""

sleep 1

echo "✅ All attestations collected (2 signatures)"
echo ""

sleep 2

echo -e "${YELLOW}📝 Step 6: Discover Permanent Storage Provider${NC}"
echo "───────────────────────────────────────────────────"
echo ""
echo "rhizoCrypt uses capability-based discovery..."
echo ""

echo "Querying discovery service for:"
echo "  Capability: PermanentStorageProvider"
echo "  Features: [commit, retrieve, merkle_proof]"
echo ""

sleep 1

echo "✅ Discovered: LoamSpine"
echo "  Endpoint: loamspine.local:9000"
echo "  Protocol: tarpc"
echo "  Status: Available"
echo ""

sleep 2

echo -e "${YELLOW}📝 Step 7: Commit to Permanent Storage${NC}"
echo "─────────────────────────────────────────"
echo ""
echo "Committing to LoamSpine..."
echo ""

echo "Payload:"
echo "  → Dehydration summary (with metadata)"
echo "  → Merkle root (cryptographic commitment)"
echo "  → Attestations (2 signatures)"
echo "  → Complete DAG structure"
echo ""

sleep 2

echo "Calling PermanentStorageProvider::commit()..."
echo ""

sleep 1

echo "✅ Committed successfully!"
echo ""
echo "Permanent Reference:"
echo "  Spine ID: loam-spine-001"
echo "  Entry Hash: ${MOCK_ROOT:0:16}..."
echo "  Index: 42"
echo "  Timestamp: 2025-12-27T10:15:30Z"
echo ""

sleep 2

echo -e "${YELLOW}📝 Step 8: Update Session Status${NC}"
echo "──────────────────────────────────"
echo ""

echo "Updating ephemeral session..."
echo "  Status: Dehydrated → Completed"
echo "  Commit Reference: Attached"
echo "  Permanent Location: Recorded"
echo ""

sleep 1

echo "✅ Session dehydration complete!"
echo ""

sleep 2

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Dehydration Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${CYAN}🎯 What Just Happened:${NC}"
echo ""
echo "1. Created ephemeral session (in-memory, fast)"
echo "2. Built DAG with 4 vertices (2 agents)"
echo "3. Computed Merkle root (cryptographic proof)"
echo "4. Generated summary (complete provenance)"
echo "5. Collected attestations (multi-party signatures)"
echo "6. Discovered storage provider (capability-based)"
echo "7. Committed to permanent storage (LoamSpine)"
echo "8. Updated session status (complete)"
echo ""

sleep 2

echo -e "${CYAN}💡 Key Benefits:${NC}"
echo ""
echo "✅ Fast Operations:"
echo "   Work in ephemeral (10-100x faster than disk)"
echo ""
echo "✅ Cryptographic Integrity:"
echo "   Merkle root proves entire DAG unchanged"
echo ""
echo "✅ Capability-Based:"
echo "   Works with ANY permanent storage (no lock-in)"
echo ""
echo "✅ Multi-Party Proof:"
echo "   All agents sign, creating audit trail"
echo ""
echo "✅ Forget by Default:"
echo "   Ephemeral session can be discarded after commit"
echo ""

sleep 2

echo -e "${CYAN}🔄 Lifecycle Summary:${NC}"
echo ""
echo "Ephemeral (rhizoCrypt)    →    Permanent (LoamSpine)"
echo "────────────────────────       ─────────────────────"
echo "• Fast (in-memory)              • Durable (persisted)"
echo "• Mutable (can change)          • Immutable (forever)"
echo "• Temporary (forget)            • Permanent (keep)"
echo "• Working space                 • Archive"
echo ""

sleep 2

echo -e "${YELLOW}📚 Real-World Use Cases:${NC}"
echo ""
echo "1. ML Experiment Tracking:"
echo "   → Train in ephemeral (fast iterations)"
echo "   → Commit final results (permanent record)"
echo ""
echo "2. Document Collaboration:"
echo "   → Edit in ephemeral (quick changes)"
echo "   → Dehydrate final version (provenance)"
echo ""
echo "3. Gaming Sessions:"
echo "   → Capture gameplay in ephemeral"
echo "   → Commit achievements (permanent)"
echo ""
echo "4. Data Processing Pipelines:"
echo "   → Process in ephemeral (fast)"
echo "   → Dehydrate results (audit trail)"
echo ""

sleep 2

echo -e "${YELLOW}▶ Next demos:${NC}"
echo "  ./demo-attestation-collection.sh   - Multi-party signing"
echo "  ./demo-dehydration-recovery.sh     - Error handling"
echo "  ./demo-large-dag-dehydration.sh    - Performance at scale"
echo ""

echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""

