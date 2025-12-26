#!/usr/bin/env bash
# Demo: Slice Provenance Mode - Read-Only with Complete History
# Time: 3 minutes
# Demonstrates: Query-only access with full provenance chain

set -euo pipefail

echo ""
echo "🔐 rhizoCrypt - Slice Semantics: Provenance Mode"
echo "================================================="
echo ""
echo "Goal: Read-only access with complete historical provenance"
echo ""

# Provenance mode gives read-only access to data along with its complete
# history: who created it, who modified it, when, why. Perfect for audits,
# compliance, research verification, or understanding data lineage.

echo "📚 Use Case: Regulatory Audit of Clinical Trial"
echo ""
echo "Scenario: A regulatory agency needs to audit a pharmaceutical"
echo "company's clinical trial data. They need:"
echo "  1. Read-only access (no modifications)"
echo "  2. Complete history of all changes"
echo "  3. Identity of every person who touched the data"
echo "  4. Timestamps of all events"
echo "  5. Cryptographic proof of integrity"
echo ""

sleep 2

echo "Step 1: Clinical trial data in permanent storage"
echo "-------------------------------------------------"
echo ""
echo "Pharmaceutical Company's LoamSpine:"
echo "  → Commit ID: loam-commit-clinical-trial-phase3"
echo "  → Data: Patient outcomes (500 patients, 2 years)"
echo "  → Current version: v7 (multiple revisions)"
echo "  → Owner: PharmaCorpResearch"
echo "  → Status: Submitted for FDA approval"
echo ""

sleep 2

echo "Step 2: Provenance checkout for audit"
echo "--------------------------------------"
echo ""
echo "FDA auditor requests access:"
cat << 'YAML'
slice_request:
  mode: Provenance
  source: loam-commit-clinical-trial-phase3
  requester: did:example:fda-auditor-smith"
  purpose: "Regulatory compliance audit"
  access_level: read_only
  include_history: full
  include_signatures: true
  include_metadata: true
  duration: 90_days
YAML
echo ""

sleep 2

echo "Step 3: rhizoCrypt creates provenance slice"
echo "--------------------------------------------"
echo ""
echo "✅ Provenance slice created"
echo "  → Slice ID: prov-audit-abc123"
echo "  → Session ID: session-prov-001"
echo "  → Access: READ ONLY"
echo "  → Auditor: did:example:fda-auditor-smith"
echo "  → History depth: COMPLETE (all 7 versions)"
echo ""

sleep 2

echo "📋 What the auditor can see:"
echo "  ✅ Current data (v7)"
echo "  ✅ All previous versions (v1-v6)"
echo "  ✅ Every person who edited"
echo "  ✅ All timestamps"
echo "  ✅ All edit reasons"
echo "  ✅ Cryptographic signatures"
echo ""
echo "❌ What the auditor CANNOT do:"
echo "  ❌ Modify any data"
echo "  ❌ Delete anything"
echo "  ❌ Export without logging"
echo "  ❌ Share with others"
echo ""

sleep 3

echo "Step 4: Auditor queries provenance chain"
echo "-----------------------------------------"
echo ""
echo "Query: 'Who created the initial dataset?'"
echo ""

sleep 1

echo "Provenance response:"
cat << 'YAML'
event: DatasetCreated
timestamp: 2023-01-15T09:00:00Z
agent: did:example:researcher-dr-johnson
data:
  initial_patients: 500
  study_design: "Randomized, double-blind, placebo-controlled"
  approval: "IRB-2023-001"
signature: [verified via BearDog]
vertex_id: genesis-abc123
YAML

sleep 2

echo ""
echo "Query: 'Show me all changes to patient outcome data'"
echo ""

sleep 1

echo "Provenance chain:"
cat << 'YAML'
version_history:
  - v1: 2023-01-15 - Initial data collection (Dr. Johnson)
  - v2: 2023-06-20 - Added 6-month follow-up (Dr. Johnson)
  - v3: 2023-09-10 - Corrected data entry error (Data Manager Lee)
    change: "Patient 247 dosage corrected: 100mg → 50mg"
    reason: "Source document review found error"
  - v4: 2023-12-15 - Added 12-month follow-up (Dr. Johnson)
  - v5: 2024-03-20 - Statistical analysis added (Statistician Chen)
  - v6: 2024-06-10 - Adverse events updated (Safety Officer Kim)
  - v7: 2024-09-01 - Final dataset for submission (Dr. Johnson)
    
all_changes_verified: true
merkle_root_per_version: [included]
YAML

sleep 3

echo ""
echo "✅ Complete provenance chain available"
echo "✅ Every change identified and explained"
echo ""

sleep 2

echo "Step 5: Auditor investigates suspicious change"
echo "-----------------------------------------------"
echo ""
echo "Auditor notices v3 correction and wants details:"
echo ""
echo "Query: 'Show me everything about the v3 dosage correction'"
echo ""

sleep 1

echo "Detailed provenance:"
cat << 'YAML'
event: DataCorrected
timestamp: 2023-09-10T14:30:00Z
agent: did:example:data-manager-lee
change:
  patient_id: 247
  field: dosage
  old_value: "100mg"
  new_value: "50mg"
reason: "Source document review identified data entry error"
evidence:
  - source_document: "CRF-247-baseline.pdf"
  - review_date: "2023-09-08"
  - reviewer: did:example:quality-officer-garcia
  - approval: did:example:principal-investigator-dr-johnson
signatures:
  - data_manager: [BearDog verified]
  - quality_officer: [BearDog verified]
  - principal_investigator: [BearDog verified]
audit_trail:
  - "Quality Officer Garcia flagged discrepancy: 2023-09-08"
  - "Source documents reviewed: 2023-09-09"
  - "Correction approved by PI: 2023-09-10 10:00:00Z"
  - "Data corrected by Manager Lee: 2023-09-10 14:30:00Z"
merkle_proof: [complete, verified]
YAML

sleep 3

echo ""
echo "✅ Complete audit trail for suspicious change"
echo "✅ Three signatures verify legitimacy"
echo "✅ Merkle proof confirms no tampering"
echo ""

sleep 2

echo "Step 6: Cryptographic verification"
echo "-----------------------------------"
echo ""
echo "Auditor verifies data integrity:"
echo ""
echo "Running cryptographic checks..."
echo "  → Verifying Merkle root for v7..."
echo "  → Checking all signatures..."
echo "  → Validating hash chain..."
echo ""

sleep 2

echo "✅ Merkle root verified: [219, 145, 88, 33, 201, ...]"
echo "✅ All 7 versions: Hash chain intact"
echo "✅ All 12 signatures: Verified via BearDog"
echo "✅ No tampering detected"
echo ""

sleep 1

echo "Auditor's conclusion:"
echo "  'Data integrity confirmed. All changes properly documented"
echo "   and approved. No evidence of scientific misconduct.'"
echo ""

sleep 2

echo "Step 7: Provenance for specific findings"
echo "-----------------------------------------"
echo ""
echo "FDA wants to trace specific clinical finding:"
echo ""
echo "Query: 'Show provenance for the 15% efficacy claim'"
echo ""

sleep 1

echo "Provenance chain for efficacy claim:"
cat << 'YAML'
claim: "15% improvement in primary endpoint"
provenance:
  - data_source: loam-commit-clinical-trial-phase3
  - analysis: "Statistical analysis v5 by Statistician Chen"
  - method: "Intent-to-treat analysis, ANCOVA"
  - patients: 500 (250 treatment, 250 placebo)
  - result: "15.3% improvement (p < 0.001)"
  - peer_review: "Reviewed by Dr. Williams (2024-04-10)"
  - approval: "Approved by PI Dr. Johnson (2024-04-15)"
  
verification_chain:
  - Raw data: v1-v4 (Dr. Johnson)
  - Statistical analysis: v5 (Statistician Chen)
  - Independent verification: External Statistician
  - Final approval: Principal Investigator
  
all_steps_signed: true
merkle_proof: verified
reproducible: true
YAML

sleep 3

echo ""
echo "✅ Complete provenance for efficacy claim"
echo "✅ Auditor can trace from raw data → final claim"
echo "✅ All steps verified and signed"
echo ""

sleep 2

echo "Step 8: Audit complete, slice expires"
echo "--------------------------------------"
echo ""
echo "After 90 days, provenance slice expires:"
echo ""
echo "Vertex final:"
echo "  → Event: ProvenanceAccessExpired"
echo "  → Auditor: did:example:fda-auditor-smith"
echo "  → Duration used: 45 days (of 90 allowed)"
echo "  → Queries made: 247"
echo "  → Violations: 0"
echo "  → Modifications attempted: 0 (none allowed)"
echo ""

sleep 1

echo "🔒 Provenance slice terminated"
echo "  → Auditor's access revoked"
echo "  → Original data: Unchanged in LoamSpine"
echo "  → Audit log: Preserved"
echo ""

sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Key Insights: Provenance Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Read-Only Access:"
echo "   Query anything, modify nothing. Perfect for audits."
echo ""
echo "✅ Complete History:"
echo "   Every change, every person, every reason preserved."
echo ""
echo "✅ Cryptographic Proof:"
echo "   Merkle proofs ensure data integrity throughout history."
echo ""
echo "✅ Signature Verification:"
echo "   All agent signatures verified via BearDog."
echo ""
echo "✅ Audit Trail:"
echo "   Who accessed what when - full compliance logging."
echo ""
echo "✅ Best For:"
echo "   • Regulatory audits (FDA, compliance)"
echo "   • Research verification (peer review)"
echo "   • Data lineage tracking (where did this come from?)"
echo "   • Forensic investigation (what happened?)"
echo "   • Compliance reporting (prove history)"
echo "   • Scientific reproducibility (trace every step)"
echo ""
echo "❌ NOT For:"
echo "   • Modifications (use Copy, Loan, etc.)"
echo "   • Collaborative editing (use Mirror mode)"
echo "   • Temporary work (use Loan mode)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Real-World Examples:"
echo ""
echo "1. FDA Drug Approval:"
echo "   - Auditor gets provenance slice of clinical trial"
echo "   - Reads all data, sees complete history"
echo "   - Verifies every change was authorized"
echo "   - Cannot modify (read-only)"
echo ""
echo "2. Scientific Paper Review:"
echo "   - Peer reviewer gets provenance of research data"
echo "   - Can verify all analysis steps"
echo "   - Traces results back to raw data"
echo "   - Checks reproducibility"
echo ""
echo "3. Legal Discovery:"
echo "   - Attorney gets provenance of business records"
echo "   - Sees who created/modified documents"
echo "   - Establishes timeline of events"
echo "   - Cryptographic proof of authenticity"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 All 6 Slice Modes Complete!"
echo ""
echo "   • Copy: Full ownership transfer"
echo "   • Loan: Temporary with auto-return"
echo "   • Consignment: Conditional ownership"
echo "   • Escrow: Multi-party conditional"
echo "   • Mirror: Real-time bidirectional sync"
echo "   • Provenance: Read-only with history ✅"
echo ""
echo "Each mode serves different use cases. Choose based on:"
echo "   - Ownership needs"
echo "   - Access duration"
echo "   - Modification rights"
echo "   - Collaboration requirements"
echo "   - Compliance obligations"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Demo complete! Provenance mode enables trustworthy auditing."
echo ""
echo "🎉 ALL SLICE SEMANTICS COMPLETE! You now understand rhizoCrypt's"
echo "   flexible data access patterns for any scenario."
echo ""

