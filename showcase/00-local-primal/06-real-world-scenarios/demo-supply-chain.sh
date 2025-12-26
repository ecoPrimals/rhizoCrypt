#!/usr/bin/env bash
# Demo: Real-World Scenario - Supply Chain with Slice Semantics
# Time: 6 minutes
# Demonstrates: Farm-to-table tracking using all 6 slice modes

set -euo pipefail

echo ""
echo "🌾 rhizoCrypt Real-World: Supply Chain Provenance"
echo "================================================="
echo ""
echo "Scenario: Track organic produce from farm to consumer"
echo ""

sleep 2

echo "📖 The Story"
echo "------------"
echo ""
echo "An organic apple travels from:"
echo "  1. Farm (grows and harvests)"
echo "  2. Processor (washes and packages)"
echo "  3. Distributor (stores and ships)"
echo "  4. Retailer (displays and sells)"
echo "  5. Consumer (buys and consumes)"
echo ""
echo "Each step uses different slice semantics!"
echo ""

sleep 3

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 1: Farm Creates Initial Record (Genesis)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Farmer creates product record in LoamSpine:"
echo ""
echo "Initial Record:"
cat << 'YAML'
product: Organic Honeycrisp Apples
batch: FARM-2025-001
quantity: 1000 lbs
farm: Green Valley Organic Farm
farmer: did:example:farmer-john
certifications: [USDA_Organic, Non_GMO]
planted: 2024-03-15
harvest: 2025-09-20
quality: Grade A
location: lat:45.5152, lon:-122.6784
YAML

echo ""
echo "✅ Committed to LoamSpine: loam-commit-apples-batch-001"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 2: Processor Uses LOAN Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Processor borrows data temporarily for washing/packaging:"
echo ""
echo "Loan Request:"
cat << 'YAML'
mode: Loan
from: loam-commit-apples-batch-001
borrower: did:example:processor-abc
duration: 2_hours
reason: "Wash and package apples"
operations: [Read, AddProcessingData]
YAML

sleep 2

echo ""
echo "🔄 Processor checks out loan..."
echo ""
echo "Processing events added:"
echo "  → Washed: 2025-09-21T06:00:00Z"
echo "  → Sorted: Removed 50 lbs (bruised)"
echo "  → Packaged: 950 lbs in 5 lb bags"
echo "  → Quality inspection: Passed"
echo "  → Processor signature: [verified]"
echo ""

sleep 2

echo "✅ Loan returned automatically after 2 hours"
echo "✅ Processing data added to permanent record"
echo "✅ Original farm data: Unchanged"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 3: Distributor Uses CONSIGNMENT Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Distributor takes consignment (conditional ownership):"
echo ""
echo "Consignment Terms:"
cat << 'YAML'
mode: Consignment
from: did:example:processor-abc
to: did:example:distributor-xyz
conditions:
  temperature_controlled: true
  max_storage_days: 14
  organic_certification: must_maintain
  report_location: daily
payment: "Pay processor after retail sale"
duration: 30_days
YAML

sleep 2

echo ""
echo "🚛 Distributor takes possession..."
echo ""
echo "Storage & transport events:"
echo "  → Cold storage: 34°F (✅ compliant)"
echo "  → Location Day 1: Warehouse A"
echo "  → Location Day 2: In transit"
echo "  → Location Day 3: Regional hub"
echo "  → Storage time: 5 days (✅ under 14 day limit)"
echo ""

sleep 2

echo "✅ Consignment terms met"
echo "✅ Daily location reports filed"
echo "✅ Temperature maintained throughout"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 4: Retailer Uses ESCROW Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Retailer purchases via escrow (payment + quality check):"
echo ""
echo "Escrow Terms:"
cat << 'YAML'
mode: Escrow
parties:
  seller: did:example:distributor-xyz
  buyer: did:example:retailer-freshmart
  arbiter: did:example:quality-inspector
conditions:
  - payment_received: true
  - quality_inspection_passed: true
  - both_parties_approved: true
price: $4500
YAML

sleep 2

echo ""
echo "🔍 Quality inspector checks apples..."
echo "  → Visual inspection: ✅ Grade A"
echo "  → Organic cert verified: ✅ Valid"
echo "  → No bruising: ✅ Passed"
echo "  → Quality score: 95/100"
echo ""

sleep 1

echo "💰 Payment clears..."
echo "  → Amount: $4500"
echo "  → Status: Confirmed"
echo ""

sleep 1

echo "✅ All escrow conditions met"
echo "✅ Ownership transferred to retailer"
echo "✅ Distributor paid"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 5: Consumer Uses PROVENANCE Mode"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Consumer scans QR code, gets full provenance:"
echo ""
echo "Provenance Query:"
cat << 'YAML'
mode: Provenance
request: "Show me where these apples came from"
consumer: did:example:consumer-sarah
access: read_only
YAML

sleep 2

echo ""
echo "📱 Consumer's phone displays:"
cat << 'TEXT'
╔═══════════════════════════════════════╗
║  🍎 YOUR APPLE'S JOURNEY 🍎           ║
╚═══════════════════════════════════════╝

🌾 Farm: Green Valley Organic Farm
   Farmer: John (certified organic)
   Planted: March 15, 2024
   Harvested: September 20, 2025
   Location: Oregon, USA
   Certifications: ✅ USDA Organic, Non-GMO

🏭 Processor: ABC Processing
   Processed: September 21, 2025
   Quality: Grade A (95/100)
   Sorted: Removed bruised apples
   Packaged: September 21, 2025

🚛 Distributor: XYZ Distribution
   Storage: Cold (34°F maintained)
   Duration: 5 days
   Quality maintained: ✅ Yes

🏪 Retailer: FreshMart
   Received: September 26, 2025
   Display: Refrigerated section
   Organic certified: ✅ Verified

✅ Complete chain of custody
✅ All certifications valid
✅ Temperature controlled throughout
✅ 6 days farm-to-shelf

VERIFIED BY: Cryptographic proof ✓
TEXT

sleep 3

echo ""
echo "✅ Consumer has complete provenance"
echo "✅ Can verify every claim"
echo "✅ Cryptographic proof of authenticity"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 6: Retailer Uses MIRROR Mode for Inventory"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Retailer keeps inventory synced in real-time:"
echo ""
echo "Mirror Setup:"
cat << 'YAML'
mode: Mirror
source: loam-commit-apples-batch-001
mirror_to: retailer-inventory-system
sync: bidirectional, real_time
reason: "Live inventory tracking"
YAML

sleep 2

echo ""
echo "📊 Real-time inventory updates:"
echo "  Day 1: 190 bags on shelf"
echo "  Day 2: 145 bags (45 sold)"
echo "  Day 3: 87 bags (58 sold)"
echo "  Day 4: 23 bags (64 sold)"
echo "  Day 5: 0 bags (23 sold) → SOLD OUT"
echo ""
echo "All updates synced to LoamSpine in real-time!"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 7: Regulator Uses PROVENANCE Mode (Audit)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "USDA auditor investigates organic certification:"
echo ""
echo "Audit Query:"
cat << 'YAML'
mode: Provenance
query: "Verify organic certification for batch FARM-2025-001"
auditor: did:example:usda-inspector
YAML

sleep 2

echo ""
echo "Complete audit trail provided:"
cat << 'YAML'
organic_certification_chain:
  farm:
    certification: USDA_Organic
    cert_number: ORG-12345
    issued: 2023-01-15
    valid_through: 2026-01-15
    verified_by: USDA
    
  processing:
    facility: ABC Processing
    organic_certified: true
    cert_number: PROC-67890
    no_contamination: verified
    
  distribution:
    organic_handling: maintained
    separation: from conventional products
    storage: organic-only warehouse
    
  retail:
    organic_section: yes
    labeling: compliant
    no_commingling: verified
    
cryptographic_verification:
  all_signatures: verified
  chain_intact: true
  no_tampering: confirmed
  
audit_result: COMPLIANT
YAML

sleep 3

echo ""
echo "✅ Organic certification verified"
echo "✅ Complete chain of custody proven"
echo "✅ No breaks in organic handling"
echo ""

sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 All 6 Slice Modes Used in Supply Chain"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Genesis (Farm):"
echo "   Creates initial product record in LoamSpine"
echo ""
echo "✅ LOAN (Processor):"
echo "   Borrows temporarily for processing, returns data"
echo ""
echo "✅ CONSIGNMENT (Distributor):"
echo "   Takes possession with conditions, reports daily"
echo ""
echo "✅ ESCROW (Retailer Purchase):"
echo "   Multi-party transaction with quality check"
echo ""
echo "✅ PROVENANCE (Consumer):"
echo "   Read-only access to complete journey"
echo ""
echo "✅ MIRROR (Retailer Inventory):"
echo "   Real-time sync for inventory management"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Key Insights:"
echo ""
echo "Different stages need different semantics:"
echo "  • Processing: Temporary access (Loan)"
echo "  • Distribution: Conditional ownership (Consignment)"
echo "  • Purchase: Multi-party trust (Escrow)"
echo "  • Consumer: Transparency (Provenance)"
echo "  • Inventory: Real-time sync (Mirror)"
echo ""
echo "rhizoCrypt's 6 slice modes handle all scenarios!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 Why This Matters:"
echo ""
echo "Supply chains are complex with many participants."
echo "Each participant needs different types of access:"
echo "  • Some need temporary access"
echo "  • Some need conditional ownership"
echo "  • Some need read-only transparency"
echo "  • Some need real-time updates"
echo ""
echo "rhizoCrypt provides the right semantic for each role."
echo "Complete provenance from farm to table, cryptographically proven."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Demo complete! Supply chain with all 6 slice modes."
echo ""
echo "🎉 ALL REAL-WORLD SCENARIOS COMPLETE!"
echo "   You now understand rhizoCrypt in production contexts."
echo ""

