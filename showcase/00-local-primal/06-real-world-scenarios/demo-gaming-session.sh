#!/usr/bin/env bash
# Demo: Real-World Scenario - Gaming Session with ML Training
# Time: 5 minutes
# Demonstrates: Complete workflow capturing player actions → ML training → commit

set -euo pipefail

echo ""
echo "🎮 rhizoCrypt Real-World: Gaming + ML Pipeline"
echo "=============================================="
echo ""
echo "Scenario: Capture a gaming session where AI learns from player behavior"
echo ""

sleep 2

echo "📖 The Story"
echo "------------"
echo ""
echo "You're building an AI-powered game that learns from players."
echo "Each gaming session needs to:"
echo "  1. Capture all player actions in a DAG"
echo "  2. Track AI agent contributions (with DIDs)"
echo "  3. Train ML model on player behavior"
echo "  4. Commit only results to permanent storage"
echo "  5. Forget intermediate data (privacy!)"
echo ""
echo "rhizoCrypt is perfect for this!"
echo ""

sleep 3

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 1: Session Creation & Player Actions"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Creating gaming session..."
echo "  → Session ID: session-game-player123-20251226"
echo "  → Player: player123"
echo "  → Game: MysteryQuest"
echo "  → Started: 2025-12-26T16:00:00Z"
echo ""

sleep 2

echo "Capturing player actions as DAG vertices:"
echo ""
echo "Vertex 1 (Genesis):"
echo "  → Event: GameStart"
echo "  → Data: {player: 'player123', level: 1}"
echo "  → Hash: abc123..."
echo ""

sleep 1

echo "Vertex 2:"
echo "  → Event: PlayerMove"
echo "  → Data: {x: 10, y: 20, direction: 'north'}"
echo "  → Parent: abc123..."
echo "  → Hash: def456..."
echo ""

sleep 1

echo "Vertex 3:"
echo "  → Event: ItemPickup"
echo "  → Data: {item: 'magic_sword', value: 100}"
echo "  → Parent: def456..."
echo "  → Hash: ghi789..."
echo ""

sleep 1

echo "Vertex 4:"
echo "  → Event: EnemyEncounter"
echo "  → Data: {enemy: 'dragon', hp: 500}"
echo "  → Parent: ghi789..."
echo "  → Hash: jkl012..."
echo ""

sleep 1

echo "Vertex 5:"
echo "  → Event: CombatAction"
echo "  → Data: {action: 'attack', damage: 75}"
echo "  → Parent: jkl012..."
echo "  → Hash: mno345..."
echo ""

sleep 2

echo "✅ 15 minutes of gameplay captured (100+ vertices in DAG)"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 2: AI Agent Enters (Multi-Agent Session)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "AI agent joins the session to provide hints:"
echo "  → Agent DID: did:example:game-ai-assistant"
echo "  → Role: Adaptive difficulty controller"
echo ""

sleep 1

echo "Vertex 101 (AI Agent):"
echo "  → Event: AIRecommendation"
echo "  → Data: {hint: 'Try fire magic against dragon', confidence: 0.85}"
echo "  → Agent: did:example:game-ai-assistant"
echo "  → Signature: [cryptographic signature]"
echo "  → Parent: mno345..."
echo "  → Hash: pqr678..."
echo ""

sleep 2

echo "✅ Multi-agent session: Player + AI both contributing"
echo "✅ AI contributions are signed (provenance!)"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 3: ML Training on Player Behavior"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "ToadStool (compute primal) starts training on session data:"
echo ""
echo "Training event captured in DAG:"
echo "Vertex 102:"
echo "  → Event: MLTrainingStart"
echo "  → Data: {model: 'player_behavior_predictor', session: 'session-game-...'}"
echo "  → Agent: did:example:toadstool-ml"
echo "  → GPU: NVIDIA_RTX_4090"
echo "  → Hash: stu901..."
echo ""

sleep 2

echo "Training progress (captured as vertices):"
echo "  Epoch 1/10: Loss = 0.85"
echo "  Epoch 5/10: Loss = 0.32"
echo "  Epoch 10/10: Loss = 0.12"
echo ""

sleep 1

echo "Vertex 112:"
echo "  → Event: MLTrainingComplete"
echo "  → Data: {final_loss: 0.12, accuracy: 0.94, epochs: 10}"
echo "  → Model checkpoint: [stored in NestGate]"
echo "  → Hash: vwx234..."
echo ""

sleep 2

echo "✅ ML training complete and tracked in DAG"
echo "✅ GPU provenance captured (hardware-level attribution)"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 4: Merkle Proof for Integrity"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Computing Merkle tree for entire session..."
echo "  → 112 vertices in DAG"
echo "  → Computing Merkle root..."
echo ""

sleep 1

echo "✅ Merkle root: [189, 234, 91, 47, 128, ...]"
echo ""
echo "This root proves:"
echo "  • All 112 vertices are part of this session"
echo "  • Player actions → AI hints → ML training are linked"
echo "  • Any tampering would change the root"
echo "  • Cryptographic integrity guaranteed"
echo ""

sleep 2

echo "Generating proof for 'AI Recommendation' vertex:"
echo "  → Proof size: 7 siblings"
echo "  → Can verify vertex belongs without revealing all data"
echo "  → Zero-knowledge proof of membership"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 5: Dehydration (Selective Permanence)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Session ends. Time to dehydrate (commit to LoamSpine):"
echo ""

cat << 'YAML'
dehydration_request:
  session: session-game-player123-20251226
  commit_what: selective
  include:
    - ML model checkpoint (vwx234...)
    - Training metrics (accuracy, loss)
    - AI recommendations (pqr678...)
    - Game outcome (player won)
  exclude:
    - Individual player moves (privacy!)
    - Intermediate combat actions
    - Exploratory actions
  merkle_root: [189, 234, 91, 47, 128, ...]
  attestations:
    - agent: did:example:game-ai-assistant
      signature: [signature]
    - agent: did:example:toadstool-ml
      signature: [signature]
YAML

sleep 3

echo ""
echo "✅ Dehydration complete!"
echo "  → Committed to LoamSpine: loam-commit-game-session-xyz"
echo "  → Contains: ML model + metrics + AI contributions"
echo "  → Does NOT contain: Individual player actions (privacy!)"
echo "  → Merkle root: Proves integrity"
echo "  → Attestations: AI and ML agent signed off"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 6: Ephemeral Memory Forgotten"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "rhizoCrypt forgets the session (Philosophy of Forgetting):"
echo "  ❌ Player moves: DELETED"
echo "  ❌ Combat actions: DELETED"
echo "  ❌ Exploration data: DELETED"
echo "  ❌ Intermediate states: DELETED"
echo "  ✅ Only committed results remain in LoamSpine"
echo ""

sleep 1

echo "Memory freed:"
echo "  → 112 vertices: DELETED"
echo "  → Session state: DELETED"
echo "  → DAG structure: DELETED"
echo "  → ~50MB RAM: FREED"
echo ""

sleep 2

echo "Why this matters:"
echo "  🔒 Player privacy preserved (individual actions forgotten)"
echo "  💾 Storage optimized (only results kept)"
echo "  ⚡ Memory freed (ephemeral by default)"
echo "  📜 Results provable (Merkle root + attestations)"
echo ""

sleep 3

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 7: Later Provenance Query"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "One month later, someone asks:"
echo "'Where did this ML model come from?'"
echo ""

sleep 1

echo "Query to SweetGrass (provenance primal):"
cat << 'YAML'
provenance_query:
  model: player_behavior_predictor_v2
  question: "What gaming sessions trained this model?"
YAML

sleep 2

echo ""
echo "SweetGrass response:"
cat << 'YAML'
provenance_chain:
  - commit: loam-commit-game-session-xyz
    session: session-game-player123-20251226
    merkle_root: [189, 234, 91, 47, 128, ...]
    agents:
      - player: player123 (actions captured)
      - ai: did:example:game-ai-assistant (recommendations)
      - ml: did:example:toadstool-ml (training)
    training_result:
      accuracy: 0.94
      loss: 0.12
      epochs: 10
    gpu: NVIDIA_RTX_4090
    verified: true
YAML

sleep 3

echo ""
echo "✅ Complete provenance chain available"
echo "✅ Can prove: Player actions → AI hints → ML training → This model"
echo "✅ Cannot see: Individual player actions (privacy preserved!)"
echo "✅ Cryptographic proof: Merkle root verifies integrity"
echo ""

sleep 3

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 What rhizoCrypt Enabled"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Session Capture:"
echo "   All player actions and AI contributions in one DAG"
echo ""
echo "✅ Multi-Agent Coordination:"
echo "   Player + AI + ML agent all contributing, all signed"
echo ""
echo "✅ Cryptographic Integrity:"
echo "   Merkle proofs ensure nothing was tampered with"
echo ""
echo "✅ Selective Permanence:"
echo "   Keep results, forget sensitive data (privacy!)"
echo ""
echo "✅ Complete Provenance:"
echo "   Can trace model back to gaming sessions"
echo ""
echo "✅ Ephemeral by Default:"
echo "   Memory freed, privacy preserved, storage optimized"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Key Takeaway:"
echo ""
echo "This is rhizoCrypt's superpower: Capture complex multi-agent"
echo "sessions with full provenance, commit only what matters, and"
echo "forget the rest. Privacy + Provenance + Performance."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Demo complete! This is what rhizoCrypt does best."
echo ""

