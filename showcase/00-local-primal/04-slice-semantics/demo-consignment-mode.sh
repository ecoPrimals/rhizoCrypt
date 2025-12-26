#!/usr/bin/env bash
# Demo: Slice Consignment Mode - Transfer with Conditions
# Time: 3 minutes
# Demonstrates: Transferring ownership with built-in conditions

set -euo pipefail

echo ""
echo "🔐 rhizoCrypt - Slice Semantics: Consignment Mode"
echo "=================================================="
echo ""
echo "Goal: Transfer data ownership with conditional terms"
echo ""

# Consignment is like shipping goods on consignment: ownership transfers
# but with conditions attached. Perfect for 2-party transfers where the
# receiver must meet certain obligations.

echo "📚 Use Case: Dataset Transfer with Attribution Requirement"
echo ""
echo "Scenario: A researcher wants to share their dataset with a colleague"
echo "at another university. The data can be transferred, but with conditions:"
echo "  1. Must cite original researcher in publications"
echo "  2. Cannot redistribute without permission"
echo "  3. Must report usage statistics annually"
echo ""

sleep 2

echo "Step 1: Original dataset in permanent storage"
echo "----------------------------------------------"
echo ""
echo "Researcher's LoamSpine:"
echo "  → Commit ID: loam-commit-biodiversity-study-2024"
echo "  → Data: 5 years of field observations (500GB)"
echo "  → Owner: Dr. Alice (University A)"
echo "  → License: Custom research agreement"
echo ""

sleep 2

echo "Step 2: Consignment checkout requested"
echo "---------------------------------------"
echo ""
echo "Request to rhizoCrypt:"
cat << 'YAML'
slice_request:
  mode: Consignment
  source: loam-commit-biodiversity-study-2024
  from: did:example:dr-alice-univ-a
  to: did:example:dr-bob-univ-b
  conditions:
    attribution:
      required: true
      citation: "Dataset courtesy of Dr. Alice, University A"
    redistribution:
      allowed: false
      requires_permission: true
    reporting:
      frequency: annual
      metrics: [usage_count, publications, insights]
  duration: 2_years
  auto_revoke_if_breached: true
  reason: "Collaborative biodiversity research"
YAML
echo ""

sleep 2

echo "Step 3: rhizoCrypt creates consignment"
echo "---------------------------------------"
echo ""
echo "✅ Consignment established"
echo "  → Consignment ID: consign-abc123"
echo "  → Session ID: session-consign-001"
echo "  → From: Dr. Alice (original owner)"
echo "  → To: Dr. Bob (new custodian)"
echo ""
echo "📋 Conditions encoded in DAG:"
echo "Vertex 1 (Genesis):"
echo "  → Event: ConsignmentCreated"
echo "  → Conditions: [attribution, no-redistribution, annual-reporting]"
echo "  → Duration: 2 years"
echo "  → Auto-revoke: true"
echo ""

sleep 2

echo "🔄 Ownership transferred (with conditions):"
echo "  → Dr. Bob now has access to full dataset"
echo "  → Original in LoamSpine: Still owned by Dr. Alice"
echo "  → Dr. Bob's copy: Bound by conditions"
echo "  → rhizoCrypt tracks compliance"
echo ""

sleep 2

echo "Step 4: Dr. Bob uses the data (compliant)"
echo "------------------------------------------"
echo ""
echo "Dr. Bob starts research with proper attribution:"
echo ""
echo "Vertex 2:"
echo "  → Event: DataUsage"
echo "  → Agent: did:example:dr-bob-univ-b"
echo "  → Data: {purpose: 'species distribution analysis', attribution: included}"
echo "  → Condition check: ✅ Attribution present"
echo ""

sleep 1

echo "Vertex 3:"
echo "  → Event: PublicationCreated"
echo "  → Agent: did:example:dr-bob-univ-b"
echo "  → Data: {title: 'Climate Impact Study', citation: 'Dataset courtesy of Dr. Alice'}"
echo "  → Condition check: ✅ Citation correct"
echo ""

sleep 2

echo "✅ Dr. Bob is compliant with conditions"
echo "✅ Consignment remains active"
echo ""

sleep 2

echo "Step 5: Attempted violation detected"
echo "-------------------------------------"
echo ""
echo "Dr. Bob's student tries to share dataset with third party:"
echo ""
echo "Vertex 4:"
echo "  → Event: RedistributionAttempted"
echo "  → Agent: did:example:student-univ-b"
echo "  → Data: {target: 'univ-c-researcher', permission: false}"
echo "  → Condition check: ❌ VIOLATION (no redistribution allowed)"
echo ""

sleep 1

echo "🚨 rhizoCrypt detects violation:"
echo ""
echo "Vertex 5:"
echo "  → Event: ConditionViolation"
echo "  → Violation: redistribution_without_permission"
echo "  → Timestamp: 2025-12-26T15:30:00Z"
echo "  → Auto-action: NOTIFY_OWNER"
echo ""

sleep 2

echo "📧 Notification sent to Dr. Alice:"
cat << 'TEXT'
VIOLATION ALERT
Consignment: consign-abc123
Violator: Student at University B
Violation: Attempted redistribution without permission
Action: Redistribution blocked, owner notified
Status: Consignment still active (warning issued)
TEXT
echo ""

sleep 2

echo "✅ Violation blocked by rhizoCrypt"
echo "✅ Original owner notified"
echo "✅ Audit trail complete"
echo ""

sleep 2

echo "Step 6: Annual reporting (compliance)"
echo "--------------------------------------"
echo ""
echo "After 1 year, Dr. Bob submits required report:"
echo ""
echo "Vertex 6:"
echo "  → Event: ComplianceReport"
echo "  → Agent: did:example:dr-bob-univ-b"
echo "  → Data:"
cat << 'YAML'
      usage_count: 147
      publications: 
        - "Climate Impact on Species Distribution (2025)"
        - "Biodiversity Trends in Temperate Forests (2025)"
      insights:
        - "Discovered correlation between temperature and migration"
        - "Identified 3 new indicator species"
      attribution_compliance: 100%
YAML
echo "  → Condition check: ✅ Report submitted on time"
echo ""

sleep 2

echo "✅ Annual reporting requirement met"
echo "✅ Consignment remains in good standing"
echo ""

sleep 2

echo "Step 7: Consignment completion (2 years later)"
echo "-----------------------------------------------"
echo ""
echo "After 2 years, consignment terms complete:"
echo ""
echo "Vertex 10:"
echo "  → Event: ConsignmentCompleted"
echo "  → Duration: 2 years (as agreed)"
echo "  → Violations: 1 (blocked)"
echo "  → Reports submitted: 2/2"
echo "  → Overall compliance: 95%"
echo ""

sleep 1

echo "🎉 Consignment successfully completed!"
echo ""
echo "Dr. Bob's options:"
echo "  1. Request renewal (new consignment)"
echo "  2. Return data (delete copy)"
echo "  3. Request purchase (transfer full ownership)"
echo ""

sleep 2

echo "Dr. Bob requests renewal:"
echo "Vertex 11:"
echo "  → Event: ConsignmentRenewalRequest"
echo "  → Agent: did:example:dr-bob-univ-b"
echo "  → Reason: 'Research ongoing, need 2 more years'"
echo ""

sleep 1

echo "Dr. Alice approves:"
echo "Vertex 12:"
echo "  → Event: ConsignmentRenewalApproved"
echo "  → Agent: did:example:dr-alice-univ-a"
echo "  → New duration: 2 years"
echo "  → Same conditions apply"
echo ""

sleep 2

echo "✅ Consignment renewed for 2 more years"
echo ""

sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Key Insights: Consignment Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Conditional Ownership:"
echo "   Transfer ownership but enforce conditions automatically."
echo ""
echo "✅ Built-in Compliance:"
echo "   rhizoCrypt tracks and enforces all conditions."
echo ""
echo "✅ Violation Detection:"
echo "   Attempts to breach conditions are caught and blocked."
echo ""
echo "✅ Audit Trail:"
echo "   Every usage, report, violation tracked in DAG."
echo ""
echo "✅ Renewable Terms:"
echo "   Can extend or modify conditions at end of period."
echo ""
echo "✅ Best For:"
echo "   • Research data sharing (cite + report)"
echo "   • Licensed content (attribution + restrictions)"
echo "   • Partnership agreements (usage + reporting)"
echo "   • Trial access (limited duration + terms)"
echo "   • Collaborative projects (share + obligations)"
echo ""
echo "❌ NOT For:"
echo "   • Simple transfers (use Copy mode)"
echo "   • Multi-party (use Escrow mode)"
echo "   • Temporary access (use Loan mode)"
echo "   • No conditions (use Copy mode)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Real-World Examples:"
echo ""
echo "1. Academic Data Sharing:"
echo "   - Transfer dataset to collaborator"
echo "   - Require citation in all publications"
echo "   - Report usage annually"
echo "   - Auto-revoke if not cited"
echo ""
echo "2. Software Beta Access:"
echo "   - Give early access to software"
echo "   - Require bug reports"
echo "   - Prohibit redistribution"
echo "   - Expire after 6 months"
echo ""
echo "3. Media Licensing:"
echo "   - License photos to publisher"
echo "   - Require attribution"
echo "   - No sublicensing"
echo "   - Report usage in campaigns"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 Comparison:"
echo "   • Copy: Unrestricted ownership"
echo "   • Loan: Temporary (must return)"
echo "   • Consignment: Ownership with conditions ✅"
echo "   • Escrow: Multi-party conditional"
echo ""
echo "✅ Demo complete! Consignment enables conditional data sharing."
echo ""

