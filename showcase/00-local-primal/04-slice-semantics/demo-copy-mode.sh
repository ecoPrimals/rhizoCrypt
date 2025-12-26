#!/usr/bin/env bash
# Demo: Slice Copy Mode - Full Ownership Transfer
# Time: 3 minutes
# Demonstrates: Checking out a slice with full ownership semantics

set -euo pipefail

echo ""
echo "🔐 rhizoCrypt - Slice Semantics: Copy Mode"
echo "=========================================="
echo ""
echo "Goal: Checkout a slice from permanent storage with full ownership"
echo ""

# This demo shows Copy mode: The most straightforward slice semantic.
# When you checkout a slice in Copy mode, you get a complete, independent
# copy of the data. You own it fully and can modify it without affecting
# the original in permanent storage.

echo "📚 Use Case: ML Model Training"
echo ""
echo "Scenario: You want to train an ML model on historical data from"
echo "permanent storage (LoamSpine). You'll modify the data (normalization,"
echo "feature engineering) so you need your own copy."
echo ""

sleep 2

echo "Step 1: Simulating permanent storage state"
echo "-------------------------------------------"
echo ""
echo "Permanent Storage (LoamSpine) contains:"
echo "  → Commit ID: loam-commit-abc123"
echo "  → Data: Historical sales data (2020-2024)"
echo "  → Size: 500MB"
echo "  → Immutable: Yes"
echo ""

sleep 2

echo "Step 2: Requesting Copy-mode checkout"
echo "--------------------------------------"
echo ""
echo "Request to rhizoCrypt:"
cat << 'YAML'
slice_request:
  mode: Copy
  source: loam-commit-abc123
  reason: "ML model training - need to normalize data"
  agent: "did:example:ml-engineer"
YAML
echo ""

sleep 2

echo "Step 3: rhizoCrypt creates ephemeral copy"
echo "-----------------------------------------"
echo ""
echo "✅ Slice checked out (Copy mode)"
echo "  → Slice ID: slice-xyz789"
echo "  → Session ID: session-001"
echo "  → Data copied to ephemeral working memory"
echo "  → Original in LoamSpine: UNCHANGED"
echo "  → You have: FULL OWNERSHIP"
echo ""

sleep 2

echo "Step 4: Working with the slice"
echo "------------------------------"
echo ""
echo "You can now:"
echo "  1. Read the data freely"
echo "  2. Modify it (normalize, transform, etc.)"
echo "  3. Add new vertices to the DAG"
echo "  4. The original in LoamSpine is unaffected"
echo ""

sleep 1

echo "Example operations:"
echo "  → Normalizing values..."
echo "  → Removing outliers..."
echo "  → Feature engineering..."
echo "  → Training ML model..."
echo ""

sleep 2

echo "Step 5: Dehydration (optional)"
echo "------------------------------"
echo ""
echo "After training, you might want to commit results:"
echo ""
echo "Dehydration request:"
cat << 'YAML'
dehydration:
  session: session-001
  results:
    - trained_model.weights
    - training_metrics.json
    - feature_importance.csv
  commit_to: LoamSpine
  original_slice: slice-xyz789 (Copy mode)
YAML
echo ""

sleep 2

echo "✅ New commit created in LoamSpine"
echo "  → Commit ID: loam-commit-def456"
echo "  → Contains: Training results"
echo "  → Links back to: loam-commit-abc123 (source data)"
echo "  → Provenance: Complete chain from raw data → results"
echo ""

sleep 1

echo "Step 6: Cleanup"
echo "---------------"
echo ""
echo "After dehydration (or when done):"
echo "  → Ephemeral copy is DELETED"
echo "  → Working memory freed"
echo "  → Only results remain in permanent storage"
echo "  → Privacy preserved (intermediate data forgotten)"
echo ""

sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Key Insights: Copy Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Full Ownership:"
echo "   You get a complete copy. Modify freely."
echo ""
echo "✅ Immutability Preserved:"
echo "   Original in LoamSpine never changes."
echo ""
echo "✅ Ephemeral by Default:"
echo "   Copy is deleted after use (unless dehydrated)."
echo ""
echo "✅ Best For:"
echo "   • ML training (need to modify data)"
echo "   • Data transformation (normalization, etc.)"
echo "   • Experiments (try things without risk)"
echo "   • Any scenario where you need your own copy"
echo ""
echo "❌ NOT For:"
echo "   • Read-only access (use Provenance mode instead)"
echo "   • Temporary access (use Loan mode instead)"
echo "   • Shared access (use Mirror mode instead)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Real-World Example:"
echo ""
echo "   A data scientist wants to train a model on customer data."
echo "   - Checkout Copy mode slice from LoamSpine"
echo "   - Normalize, engineer features, train model"
echo "   - Dehydrate results (model weights, metrics)"
echo "   - Ephemeral copy is deleted (privacy!)"
echo "   - Only results persist"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 Compare with other modes:"
echo "   • Loan: Temporary access, must return"
echo "   • Consignment: Transfer with conditions"
echo "   • Escrow: Multi-party holding"
echo "   • Mirror: Synchronized copy"
echo "   • Provenance: Read-only with history"
echo ""
echo "✅ Demo complete! Copy mode is the simplest and most flexible."
echo ""

