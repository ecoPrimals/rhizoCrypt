#!/usr/bin/env bash
# Demo: Real-World Scenario - Document Workflow with Multi-Party Provenance
# Time: 5 minutes
# Demonstrates: Contract negotiation with full audit trail and signatures

set -euo pipefail

echo ""
echo "📄 rhizoCrypt Real-World: Legal Document Workflow"
echo "=================================================="
echo ""
echo "Scenario: Multi-party contract negotiation with provenance"
echo ""

sleep 2

echo "📖 The Story"
echo "------------"
echo ""
echo "A software contract between:"
echo "  • Client (BigCorp)"
echo "  • Vendor (DevShop)"  
echo "  • Legal reviewer (Law Firm)"
echo ""
echo "Requirements:"
echo "  1. Track all edits and who made them"
echo "  2. Each party must review and approve"
echo "  3. Full audit trail for compliance"
echo "  4. Cryptographic proof of agreement"
echo "  5. Later: Query 'who approved what when?'"
echo ""
echo "rhizoCrypt captures the entire workflow!"
echo ""

sleep 3

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 1: Initial Draft (Vendor)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Creating contract session..."
echo "  → Session ID: session-contract-bigcorp-devshop"
echo "  → Document: Software Development Agreement"
echo "  → Parties: 3 (Client, Vendor, Legal)"
echo "  → Started: 2025-12-26T09:00:00Z"
echo ""

sleep 2

echo "Vendor creates initial draft:"
echo ""
echo "Vertex 1 (Genesis):"
echo "  → Event: DocumentCreated"
echo "  → Agent: did:example:devshop-sales"
echo "  → Data: {doc_id: 'contract-v0.1', sections: 12}"
echo "  → Payload: [stored in NestGate]"
echo "  → Signature: [DevShop signature]"
echo "  → Hash: abc123..."
echo ""

sleep 2

echo "✅ Initial draft created and signed by vendor"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 2: Client Review & Edits"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Client reviews draft and requests changes:"
echo ""
echo "Vertex 2:"
echo "  → Event: DocumentEdited"
echo "  → Agent: did:example:bigcorp-procurement"
echo "  → Data: {section: 'Payment Terms', change: 'Net 60 → Net 30'}"
echo "  → Parent: abc123..."
echo "  → Signature: [BigCorp signature]"
echo "  → Hash: def456..."
echo ""

sleep 1

echo "Vertex 3:"
echo "  → Event: DocumentComment"
echo "  → Agent: did:example:bigcorp-procurement"
echo "  → Data: {section: 'Liability', comment: 'Cap too high, reduce to \$1M'}"
echo "  → Parent: def456..."
echo "  → Signature: [BigCorp signature]"
echo "  → Hash: ghi789..."
echo ""

sleep 2

echo "✅ Client edits captured: Payment terms + Liability cap"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 3: Vendor Responds"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Vendor reviews client's changes:"
echo ""
echo "Vertex 4:"
echo "  → Event: DocumentEdited"
echo "  → Agent: did:example:devshop-sales"
echo "  → Data: {section: 'Payment Terms', status: 'Accepted'}"
echo "  → Parent: ghi789..."
echo "  → Signature: [DevShop signature]"
echo "  → Hash: jkl012..."
echo ""

sleep 1

echo "Vertex 5:"
echo "  → Event: DocumentEdited"
echo "  → Agent: did:example:devshop-sales"
echo "  → Data: {section: 'Liability', change: 'Compromise: \$1.5M cap'}"
echo "  → Parent: jkl012..."
echo "  → Signature: [DevShop signature]"
echo "  → Hash: mno345..."
echo ""

sleep 2

echo "✅ Vendor accepts payment change, proposes liability compromise"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 4: Legal Review"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Law firm reviews entire document:"
echo ""
echo "Vertex 6:"
echo "  → Event: LegalReview"
echo "  → Agent: did:example:lawfirm-partner"
echo "  → Data: {status: 'Reviewed', issues: [], recommendation: 'Approve'}"
echo "  → Parent: mno345..."
echo "  → Signature: [Law Firm signature]"
echo "  → Hash: pqr678..."
echo ""

sleep 2

echo "Vertex 7:"
echo "  → Event: ComplianceCheck"
echo "  → Agent: did:example:lawfirm-compliance"
echo "  → Data: {gdpr: true, ccpa: true, hipaa: 'n/a', compliant: true}"
echo "  → Parent: pqr678..."
echo "  → Signature: [Compliance Officer signature]"
echo "  → Hash: stu901..."
echo ""

sleep 2

echo "✅ Legal review complete: All compliant, recommend approval"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 5: Final Approvals (Multi-Party Signatures)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Client final approval:"
echo "Vertex 8:"
echo "  → Event: DocumentApproved"
echo "  → Agent: did:example:bigcorp-ceo"
echo "  → Data: {approved: true, authority: 'CEO', timestamp: '2025-12-26T14:00:00Z'}"
echo "  → Parent: stu901..."
echo "  → Signature: [CEO signature with BearDog HSM]"
echo "  → Hash: vwx234..."
echo ""

sleep 2

echo "Vendor final approval:"
echo "Vertex 9:"
echo "  → Event: DocumentApproved"
echo "  → Agent: did:example:devshop-founder"
echo "  → Data: {approved: true, authority: 'Founder', timestamp: '2025-12-26T14:15:00Z'}"
echo "  → Parent: vwx234..."
echo "  → Signature: [Founder signature with BearDog HSM]"
echo "  → Hash: yz0567..."
echo ""

sleep 2

echo "Legal final certification:"
echo "Vertex 10:"
echo "  → Event: LegalCertification"
echo "  → Agent: did:example:lawfirm-partner"
echo "  → Data: {certified: true, bar_number: 'CA-123456', timestamp: '2025-12-26T14:30:00Z'}"
echo "  → Parent: yz0567..."
echo "  → Signature: [Attorney signature with BearDog HSM]"
echo "  → Hash: abc890..."
echo ""

sleep 3

echo "✅ All three parties have signed off!"
echo "✅ Contract is now legally binding"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 6: Merkle Proof for Legal Validity"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Computing Merkle tree for entire negotiation..."
echo "  → 10 vertices (draft → edits → reviews → approvals)"
echo "  → 3 agent signatures (Client, Vendor, Legal)"
echo "  → Computing Merkle root..."
echo ""

sleep 1

echo "✅ Merkle root: [219, 145, 88, 33, 201, ...]"
echo ""
echo "This root cryptographically proves:"
echo "  • Complete negotiation history"
echo "  • All edits, comments, approvals linked"
echo "  • No tampering possible (hash chain)"
echo "  • Legal validity (three signatures)"
echo ""

sleep 2

echo "Generating proof for CEO approval (Vertex 8):"
echo "  → Proof size: 4 siblings"
echo "  → Can prove CEO signed without revealing full document"
echo "  → Zero-knowledge proof for compliance queries"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 7: Dehydration (Commit to Permanent Record)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Committing to LoamSpine for legal permanence:"
cat << 'YAML'
dehydration_request:
  session: session-contract-bigcorp-devshop
  commit_what: full
  include:
    - Initial draft (Vertex 1)
    - All edits (Vertices 2-5)
    - Legal review (Vertices 6-7)
    - All approvals (Vertices 8-10)
    - Final document version
  merkle_root: [219, 145, 88, 33, 201, ...]
  signatures:
    - agent: did:example:bigcorp-ceo
    - agent: did:example:devshop-founder
    - agent: did:example:lawfirm-partner
  legal_status: BINDING_CONTRACT
YAML

sleep 3

echo ""
echo "✅ Contract committed to LoamSpine"
echo "  → Commit ID: loam-commit-contract-final-abc"
echo "  → Contains: Full negotiation history"
echo "  → Signatures: All three parties"
echo "  → Merkle root: Proves integrity"
echo "  → Status: Legally binding, immutable"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 8: Provenance Query (6 Months Later)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Scenario: Audit asks 'Who approved the liability clause?'"
echo ""
echo "Query to SweetGrass (provenance primal):"
cat << 'YAML'
provenance_query:
  contract: loam-commit-contract-final-abc
  question: "Who approved the \$1.5M liability cap?"
YAML

sleep 2

echo ""
echo "SweetGrass response (complete provenance chain):"
cat << 'YAML'
provenance_chain:
  section: "Liability Cap"
  history:
    - event: "Client requested \$1M cap"
      agent: did:example:bigcorp-procurement
      timestamp: 2025-12-26T10:30:00Z
      vertex: ghi789...
      
    - event: "Vendor countered \$1.5M cap"
      agent: did:example:devshop-sales
      timestamp: 2025-12-26T11:00:00Z
      vertex: mno345...
      
    - event: "Legal reviewed and approved"
      agent: did:example:lawfirm-partner
      timestamp: 2025-12-26T13:00:00Z
      vertex: pqr678...
      
    - event: "CEO approved final contract"
      agent: did:example:bigcorp-ceo
      timestamp: 2025-12-26T14:00:00Z
      vertex: vwx234...
      signature: [BearDog HSM verified]
      
  final_approval_chain:
    - Client CEO: did:example:bigcorp-ceo ✅
    - Vendor Founder: did:example:devshop-founder ✅
    - Attorney: did:example:lawfirm-partner ✅
    
  merkle_proof: [complete, verified]
  legal_status: BINDING
YAML

sleep 3

echo ""
echo "✅ Complete answer with cryptographic proof!"
echo "✅ Can trace every decision back to source"
echo "✅ All signatures verified via BearDog"
echo "✅ Merkle proof confirms no tampering"
echo ""

sleep 3

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 What rhizoCrypt Enabled"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Complete Audit Trail:"
echo "   Every edit, comment, review captured in immutable DAG"
echo ""
echo "✅ Multi-Party Signatures:"
echo "   Client + Vendor + Legal all sign with DIDs"
echo ""
echo "✅ Cryptographic Integrity:"
echo "   Merkle root proves no post-signature tampering"
echo ""
echo "✅ Legal Compliance:"
echo "   Full provenance for audits, disputes, regulation"
echo ""
echo "✅ Selective Disclosure:"
echo "   Can prove specific facts (CEO signed) without revealing all"
echo ""
echo "✅ Permanent Record:"
echo "   Committed to LoamSpine, legally binding, immutable"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Why This Matters:"
echo ""
echo "Traditional document management:"
echo "  ❌ No cryptographic proof of who edited what"
echo "  ❌ Signatures can be disputed"
echo "  ❌ Audit trails can be altered"
echo "  ❌ No way to prove specific claims"
echo ""
echo "With rhizoCrypt:"
echo "  ✅ Immutable edit history (DAG)"
echo "  ✅ Cryptographic signatures (BearDog)"
echo "  ✅ Tamper-proof record (Merkle proofs)"
echo "  ✅ Selective disclosure (zero-knowledge proofs)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎓 Key Takeaway:"
echo ""
echo "rhizoCrypt transforms document workflows from fragile"
echo "paper trails into cryptographically-proven provenance chains."
echo "Perfect for contracts, compliance, legal, and any scenario"
echo "where 'who did what when' must be provably correct."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Demo complete! Document provenance with legal validity."
echo ""

