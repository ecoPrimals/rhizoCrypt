#!/usr/bin/env bash
# Demo: Slice Loan Mode - Temporary Access with Auto-Return
# Time: 3 minutes
# Demonstrates: Borrowing data temporarily with guaranteed return

set -euo pipefail

echo ""
echo "🔐 rhizoCrypt - Slice Semantics: Loan Mode"
echo "==========================================="
echo ""
echo "Goal: Borrow data temporarily, system ensures return"
echo ""

# Loan mode is like checking out a library book: you get temporary access,
# but the system tracks it and expects it back. Perfect for read-heavy
# operations where you don't need permanent ownership.

echo "📚 Use Case: AI Inference on Production Data"
echo ""
echo "Scenario: An AI model needs to run inference on customer data."
echo "You need the data temporarily for computation, but don't want"
echo "permanent ownership. The data should return to storage after use."
echo ""

sleep 2

echo "Step 1: Current state in permanent storage"
echo "-------------------------------------------"
echo ""
echo "Permanent Storage (LoamSpine):"
echo "  → Commit ID: loam-commit-customer-data"
echo "  → Data: Customer profiles (10,000 records)"
echo "  → Owner: Data governance team"
echo "  → Status: Locked for production use"
echo ""

sleep 2

echo "Step 2: Requesting Loan-mode checkout"
echo "--------------------------------------"
echo ""
echo "Request to rhizoCrypt:"
cat << 'YAML'
slice_request:
  mode: Loan
  source: loam-commit-customer-data
  duration: 1_hour
  reason: "AI inference - sentiment analysis"
  agent: "did:example:ai-service"
  operations: [Read, Compute]
YAML
echo ""

sleep 2

echo "Step 3: rhizoCrypt creates tracked loan"
echo "----------------------------------------"
echo ""
echo "✅ Slice loaned (Loan mode)"
echo "  → Loan ID: loan-abc789"
echo "  → Session ID: session-ai-001"
echo "  → Expires: 1 hour from now"
echo "  → Permissions: Read + Compute only"
echo "  → Auto-return: Enabled"
echo ""
echo "📋 Loan tracked in rhizoCrypt:"
echo "  → Borrower: did:example:ai-service"
echo "  → Checkout time: 2025-12-26T15:30:00Z"
echo "  → Due back: 2025-12-26T16:30:00Z"
echo "  → Status: ACTIVE"
echo ""

sleep 2

echo "Step 4: Working with the loaned slice"
echo "--------------------------------------"
echo ""
echo "Permitted operations:"
echo "  ✅ Read customer profiles"
echo "  ✅ Run sentiment analysis model"
echo "  ✅ Generate inference results"
echo "  ✅ Add results as new vertices in DAG"
echo ""
echo "Restricted operations:"
echo "  ❌ Modify original customer data (read-only)"
echo "  ❌ Transfer to another agent"
echo "  ❌ Keep beyond expiration time"
echo ""

sleep 2

echo "Running AI inference..."
echo "  → Processing record 1000/10000..."
echo "  → Processing record 5000/10000..."
echo "  → Processing record 10000/10000..."
echo "  ✅ Inference complete!"
echo ""

sleep 2

echo "Step 5: Dehydration (save results only)"
echo "----------------------------------------"
echo ""
echo "After inference, commit only the results:"
cat << 'YAML'
dehydration:
  session: session-ai-001
  results:
    - sentiment_scores.json
    - insights_summary.json
  commit_to: LoamSpine
  loan_id: loan-abc789
YAML
echo ""
echo "✅ Results committed to LoamSpine"
echo "  → New commit: loam-commit-sentiments"
echo "  → Contains: Only inference results"
echo "  → Links to: loam-commit-customer-data (source)"
echo "  → Customer data: Not copied, just referenced"
echo ""

sleep 2

echo "Step 6: Auto-return mechanism"
echo "------------------------------"
echo ""
echo "Two ways the loan returns:"
echo ""
echo "1️⃣  Manual return (after dehydration):"
echo "    → Agent signals 'done'"
echo "    → rhizoCrypt releases the slice"
echo "    → Loan status: RETURNED"
echo ""
echo "2️⃣  Auto-return (timeout):"
echo "    → 1 hour expires"
echo "    → rhizoCrypt forcibly releases slice"
echo "    → Any uncommitted work: LOST"
echo "    → Loan status: EXPIRED"
echo ""

sleep 2

echo "✅ Loan returned"
echo "  → Return time: 2025-12-26T15:55:00Z (25 minutes used)"
echo "  → Data returned to: LoamSpine (loam-commit-customer-data)"
echo "  → Results saved in: loam-commit-sentiments"
echo "  → Ephemeral copy: DELETED"
echo "  → Memory freed: 500MB"
echo ""

sleep 1

echo "Step 7: Audit trail"
echo "-------------------"
echo ""
echo "Loan record preserved for compliance:"
cat << 'YAML'
loan_audit:
  loan_id: loan-abc789
  borrower: did:example:ai-service
  data_source: loam-commit-customer-data
  checkout: 2025-12-26T15:30:00Z
  return: 2025-12-26T15:55:00Z
  duration_used: 25_minutes
  operations: [Read, Compute]
  results: loam-commit-sentiments
  status: RETURNED_SUCCESSFULLY
YAML
echo ""

sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Key Insights: Loan Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Temporary Access:"
echo "   Borrow data, use it, return it automatically."
echo ""
echo "✅ Enforced Return:"
echo "   System ensures data comes back (timeout or explicit return)."
echo ""
echo "✅ Read-Only by Default:"
echo "   Original data protected, you can compute but not modify source."
echo ""
echo "✅ Audit Trail:"
echo "   Every loan tracked: who, when, for how long, what they did."
echo ""
echo "✅ Best For:"
echo "   • AI inference (need data temporarily)"
echo "   • Analytics (read, compute, save results)"
echo "   • Batch processing (run job, return data)"
echo "   • Compliance-heavy scenarios (need audit trail)"
echo ""
echo "❌ NOT For:"
echo "   • Long-term access (use Copy or Consignment)"
echo "   • Modifying source data (use Copy mode)"
echo "   • Permanent ownership (use Consignment)"
echo "   • Shared simultaneous access (use Mirror mode)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Real-World Example:"
echo ""
echo "   A compliance officer needs to run an audit report on"
echo "   financial transactions:"
echo "   - Loan mode checkout (1 hour duration)"
echo "   - Run audit queries (read-only)"
echo "   - Generate report"
echo "   - Dehydrate report results"
echo "   - Loan auto-returns after 30 minutes"
echo "   - Full audit trail maintained"
echo "   - Original transaction data never modified"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 Comparison:"
echo "   • Copy: Full ownership, keep forever"
echo "   • Loan: Temporary access, must return ✅"
echo "   • Consignment: Transfer with conditions"
echo "   • Escrow: Multi-party holding"
echo ""
echo "✅ Demo complete! Loan mode ensures data returns home."
echo ""

