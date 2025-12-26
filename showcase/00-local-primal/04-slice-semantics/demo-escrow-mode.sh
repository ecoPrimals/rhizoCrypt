#!/usr/bin/env bash
# Demo: Slice Escrow Mode - Multi-Party Holding with Conditions
# Time: 4 minutes
# Demonstrates: Holding data in escrow until conditions are met

set -euo pipefail

echo ""
echo "🔐 rhizoCrypt - Slice Semantics: Escrow Mode"
echo "============================================="
echo ""
echo "Goal: Hold data in multi-party escrow until conditions met"
echo ""

# Escrow mode is perfect for scenarios where data must be held by a neutral
# party until specific conditions are satisfied. Think: contracts, payments,
# multi-sig authorization.

echo "📚 Use Case: Research Data Sharing with Payment"
echo ""
echo "Scenario: A pharmaceutical company wants to buy research data"
echo "from a university. The data should be held in escrow until:"
echo "  1. Payment clears"
echo "  2. Data quality verified by third party"
echo "  3. Both parties sign off"
echo ""

sleep 2

echo "Step 1: Research data in permanent storage"
echo "-------------------------------------------"
echo ""
echo "University's LoamSpine:"
echo "  → Commit ID: loam-commit-clinical-trials-2024"
echo "  → Data: Clinical trial results (10 years, 5000 patients)"
echo "  → Value: \$500,000"
echo "  → Owner: University Research Lab"
echo "  → Status: Private, not released"
echo ""

sleep 2

echo "Step 2: Escrow checkout requested"
echo "----------------------------------"
echo ""
echo "Request to rhizoCrypt:"
cat << 'YAML'
slice_request:
  mode: Escrow
  source: loam-commit-clinical-trials-2024
  parties:
    seller: did:example:university-research-lab
    buyer: did:example:pharma-company
    arbiter: did:example:data-quality-auditor
  conditions:
    - payment_received: true
    - data_verified: true
    - both_parties_approved: true
  escrow_duration: 30_days
  reason: "Research data purchase agreement"
YAML
echo ""

sleep 2

echo "Step 3: rhizoCrypt creates escrow"
echo "----------------------------------"
echo ""
echo "✅ Escrow established"
echo "  → Escrow ID: escrow-xyz789"
echo "  → Session ID: session-escrow-001"
echo "  → Parties tracked:"
echo "    - Seller: University (holds data)"
echo "    - Buyer: Pharma Co (wants data)"
echo "    - Arbiter: Auditor (verifies quality)"
echo ""
echo "📋 Conditions status:"
echo "  ❌ payment_received: false"
echo "  ❌ data_verified: false"
echo "  ❌ both_parties_approved: false"
echo ""
echo "🔒 Data state: HELD IN ESCROW"
echo "   → Seller cannot withdraw"
echo "   → Buyer cannot access yet"
echo "   → Only arbiter can inspect"
echo ""

sleep 3

echo "Step 4: Condition 1 - Payment clears"
echo "-------------------------------------"
echo ""
echo "Pharma company sends payment..."
echo "  → Amount: \$500,000"
echo "  → Transaction: blockchain-tx-abc123"
echo "  → Verified by: Smart contract"
echo ""

sleep 1

echo "Payment event added to DAG:"
echo "Vertex 50:"
echo "  → Event: EscrowPaymentReceived"
echo "  → Data: {amount: 500000, tx: 'blockchain-tx-abc123'}"
echo "  → Agent: did:example:payment-oracle"
echo "  → Signature: [verified]"
echo ""

sleep 1

echo "✅ Condition 1 satisfied: payment_received = true"
echo ""

sleep 2

echo "Step 5: Condition 2 - Data quality verification"
echo "------------------------------------------------"
echo ""
echo "Arbiter inspects the data..."
echo "  → Running quality checks..."
echo "  → Verifying patient count (5000)..."
echo "  → Checking data completeness (100%)..."
echo "  → Validating statistical methods..."
echo ""

sleep 2

echo "Verification event added to DAG:"
echo "Vertex 51:"
echo "  → Event: EscrowDataVerified"
echo "  → Data: {verified: true, quality_score: 98, issues: []}"
echo "  → Agent: did:example:data-quality-auditor"
echo "  → Signature: [verified]"
echo ""

sleep 1

echo "✅ Condition 2 satisfied: data_verified = true"
echo ""

sleep 2

echo "Step 6: Condition 3 - Both parties approve"
echo "-------------------------------------------"
echo ""
echo "Seller (University) approval:"
echo "Vertex 52:"
echo "  → Event: EscrowSellerApproval"
echo "  → Data: {approved: true, timestamp: '2025-12-26T17:00:00Z'}"
echo "  → Agent: did:example:university-research-lab"
echo "  → Signature: [verified]"
echo ""

sleep 1

echo "Buyer (Pharma) approval:"
echo "Vertex 53:"
echo "  → Event: EscrowBuyerApproval"
echo "  → Data: {approved: true, timestamp: '2025-12-26T17:05:00Z'}"
echo "  → Agent: did:example:pharma-company"
echo "  → Signature: [verified]"
echo ""

sleep 1

echo "✅ Condition 3 satisfied: both_parties_approved = true"
echo ""

sleep 2

echo "Step 7: All conditions met - Escrow releases"
echo "---------------------------------------------"
echo ""
echo "📋 Final condition check:"
echo "  ✅ payment_received: true"
echo "  ✅ data_verified: true"
echo "  ✅ both_parties_approved: true"
echo ""
echo "🎉 ALL CONDITIONS SATISFIED!"
echo ""

sleep 1

echo "rhizoCrypt releases escrow:"
echo ""
echo "Release event added to DAG:"
echo "Vertex 54:"
echo "  → Event: EscrowReleased"
echo "  → Data: {released_to: 'buyer', conditions_met: ['payment', 'verification', 'approval']}"
echo "  → Timestamp: 2025-12-26T17:10:00Z"
echo ""

sleep 2

echo "Step 8: Data transfer to buyer"
echo "-------------------------------"
echo ""
echo "✅ Data now available to buyer (Pharma Co)"
echo "  → Checkout mode: Copy (full ownership)"
echo "  → New owner: did:example:pharma-company"
echo "  → Original in LoamSpine: Still owned by University"
echo "  → Escrow complete: Success"
echo ""

sleep 2

echo "Step 9: Audit trail & dehydration"
echo "----------------------------------"
echo ""
echo "Complete escrow lifecycle committed to LoamSpine:"
cat << 'YAML'
escrow_audit:
  escrow_id: escrow-xyz789
  parties:
    seller: did:example:university-research-lab
    buyer: did:example:pharma-company
    arbiter: did:example:data-quality-auditor
  conditions_met:
    - payment_received: 2025-12-26T16:30:00Z
    - data_verified: 2025-12-26T16:45:00Z
    - both_parties_approved: 2025-12-26T17:05:00Z
  released: 2025-12-26T17:10:00Z
  data_transferred: true
  status: COMPLETED_SUCCESSFULLY
YAML

sleep 2

echo ""
echo "✅ Escrow record committed for compliance"
echo "✅ Complete provenance chain maintained"
echo "✅ All party signatures preserved"
echo ""

sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Key Insights: Escrow Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Multi-Party Trust:"
echo "   Neutral third party (rhizoCrypt) holds data until conditions met."
echo ""
echo "✅ Conditional Release:"
echo "   Data only released when ALL conditions satisfied."
echo ""
echo "✅ Complete Audit Trail:"
echo "   Every condition check, signature, event tracked in DAG."
echo ""
echo "✅ Cryptographic Proof:"
echo "   All parties sign off, Merkle proof ensures integrity."
echo ""
echo "✅ Best For:"
echo "   • Data sales (payment + verification)"
echo "   • Multi-sig authorization (corporate approvals)"
echo "   • Research collaborations (ethics + consent)"
echo "   • Regulated data sharing (compliance conditions)"
echo "   • Contract execution (milestone-based release)"
echo ""
echo "❌ NOT For:"
echo "   • Simple transfers (use Copy or Consignment)"
echo "   • Temporary access (use Loan mode)"
echo "   • Two-party trust (use Consignment)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Real-World Examples:"
echo ""
echo "1. Research Data Sales:"
echo "   - Escrow until payment + quality verification"
echo "   - Arbiter ensures data meets specifications"
echo "   - Both parties must approve final transfer"
echo ""
echo "2. Corporate M&A:"
echo "   - Sensitive documents in escrow"
echo "   - Release after due diligence + legal approval"
echo "   - Multiple executives must sign off"
echo ""
echo "3. Clinical Trial Data:"
echo "   - Patient data in escrow"
echo "   - Release only after ethics board approval"
echo "   - Both hospital and pharma approve"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 Escrow vs Other Modes:"
echo "   • Copy: Direct ownership (no conditions)"
echo "   • Loan: Temporary (no transfer of ownership)"
echo "   • Consignment: 2-party with conditions"
echo "   • Escrow: Multi-party with conditions ✅"
echo "   • Mirror: Synchronized (no escrow)"
echo ""
echo "✅ Demo complete! Escrow enables trustless multi-party data sharing."
echo ""

