#!/usr/bin/env bash
# Demo: Real-World Scenario - Multi-Agent ML Pipeline
# Time: 6 minutes
# Demonstrates: Complete ML training with provenance across multiple agents

set -euo pipefail

echo ""
echo "🤖 rhizoCrypt Real-World: Multi-Agent ML Pipeline"
echo "=================================================="
echo ""
echo "Scenario: Distributed ML training with full provenance tracking"
echo ""

sleep 2

echo "📖 The Story"
echo "------------"
echo ""
echo "A machine learning team is training a fraud detection model."
echo "The pipeline involves:"
echo "  • Data Engineer: Prepares training data"
echo "  • ML Engineer: Trains the model"
echo "  • QA Engineer: Validates model quality"
echo "  • Security Engineer: Reviews for bias"
echo "  • DevOps: Deploys to production"
echo ""
echo "rhizoCrypt captures every step with full attribution!"
echo ""

sleep 3

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 1: Data Preparation (Data Engineer)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Creating ML pipeline session..."
echo "  → Session ID: session-ml-fraud-detection-v2"
echo "  → Pipeline: Fraud Detection Model Training"
echo "  → Team: 5 agents across 3 organizations"
echo "  → Started: 2025-12-26T08:00:00Z"
echo ""

sleep 2

echo "Data Engineer Alice starts data prep:"
echo ""
echo "Vertex 1 (Genesis):"
echo "  → Event: PipelineStarted"
echo "  → Agent: did:example:data-engineer-alice"
echo "  → Data: {pipeline: 'fraud-detection-v2', data_source: 'transactions-2024'}"
echo "  → Hash: pipeline-start-abc..."
echo ""

sleep 1

echo "Vertex 2:"
echo "  → Event: DataIngestion"
echo "  → Agent: did:example:data-engineer-alice"
echo "  → Data: {records: 10_000_000, source: 'production-db'}"
echo "  → Parent: pipeline-start-abc..."
echo "  → Hash: ingest-def..."
echo ""

sleep 1

echo "Vertex 3:"
echo "  → Event: DataCleaning"
echo "  → Agent: did:example:data-engineer-alice"
echo "  → Data: {removed_nulls: 45_000, removed_duplicates: 12_000}"
echo "  → Parent: ingest-def..."
echo "  → Hash: clean-ghi..."
echo ""

sleep 1

echo "Vertex 4:"
echo "  → Event: FeatureEngineering"
echo "  → Agent: did:example:data-engineer-alice"
echo "  → Data: {features_created: 47, encoding: 'one-hot', scaling: 'standard'}"
echo "  → Parent: clean-ghi..."
echo "  → Hash: features-jkl..."
echo ""

sleep 2

echo "✅ Data preparation complete (10M records → 9.9M clean)"
echo "✅ 47 features engineered"
echo "✅ All steps signed by Alice"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 2: Model Training (ML Engineer)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "ML Engineer Bob takes over with ToadStool (compute primal):"
echo ""
echo "Vertex 5:"
echo "  → Event: TrainingStarted"
echo "  → Agent: did:example:ml-engineer-bob"
echo "  → Data: {model: 'gradient-boosting', framework: 'XGBoost'}"
echo "  → Compute: ToadStool cluster (8x GPU)"
echo "  → Parent: features-jkl..."
echo "  → Hash: train-start-mno..."
echo ""

sleep 2

echo "Training progress captured as vertices:"
echo ""
echo "Vertex 6 (Epoch 10/100):"
echo "  → Event: TrainingProgress"
echo "  → Data: {epoch: 10, loss: 0.45, accuracy: 0.78}"
echo "  → GPU: NVIDIA_A100 × 8"
echo "  → Compute time: 15 minutes"
echo ""

sleep 1

echo "Vertex 7 (Epoch 50/100):"
echo "  → Event: TrainingProgress"
echo "  → Data: {epoch: 50, loss: 0.18, accuracy: 0.94}"
echo "  → GPU: NVIDIA_A100 × 8"
echo "  → Compute time: 75 minutes total"
echo ""

sleep 1

echo "Vertex 8 (Complete):"
echo "  → Event: TrainingCompleted"
echo "  → Agent: did:example:ml-engineer-bob"
echo "  → Data: {final_loss: 0.12, final_accuracy: 0.96, epochs: 100}"
echo "  → Model checkpoint: [stored in NestGate]"
echo "  → Parent: train-start-mno..."
echo "  → Signature: [Bob's signature]"
echo "  → Hash: train-complete-pqr..."
echo ""

sleep 2

echo "✅ Model training complete: 96% accuracy"
echo "✅ Model checkpoint stored in NestGate"
echo "✅ GPU provenance tracked (ToadStool)"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 3: Quality Validation (QA Engineer)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "QA Engineer Carol validates model quality:"
echo ""
echo "Vertex 9:"
echo "  → Event: ValidationStarted"
echo "  → Agent: did:example:qa-engineer-carol"
echo "  → Data: {test_set: 'holdout-2024-q4', size: 1_000_000}"
echo "  → Parent: train-complete-pqr..."
echo "  → Hash: val-start-stu..."
echo ""

sleep 1

echo "Running comprehensive tests..."
echo "  → Precision: 0.95"
echo "  → Recall: 0.94"
echo "  → F1-score: 0.945"
echo "  → AUC-ROC: 0.98"
echo "  → False positive rate: 0.02"
echo ""

sleep 1

echo "Vertex 10:"
echo "  → Event: ValidationComplete"
echo "  → Agent: did:example:qa-engineer-carol"
echo "  → Data: {precision: 0.95, recall: 0.94, f1: 0.945, auc: 0.98}"
echo "  → Decision: PASSED"
echo "  → Parent: val-start-stu..."
echo "  → Signature: [Carol's signature]"
echo "  → Hash: val-pass-vwx..."
echo ""

sleep 2

echo "✅ Quality validation passed"
echo "✅ Model meets production thresholds"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 4: Bias Review (Security Engineer)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Security Engineer Dave checks for algorithmic bias:"
echo ""
echo "Vertex 11:"
echo "  → Event: BiasAuditStarted"
echo "  → Agent: did:example:security-engineer-dave"
echo "  → Data: {focus: 'demographic parity, equal opportunity'}"
echo "  → Parent: val-pass-vwx..."
echo "  → Hash: bias-start-yz0..."
echo ""

sleep 2

echo "Analyzing fairness metrics across demographics..."
echo "  → Gender: Parity score 0.98 (✅ Fair)"
echo "  → Age: Parity score 0.97 (✅ Fair)"
echo "  → Geography: Parity score 0.95 (✅ Fair)"
echo "  → Income level: Parity score 0.96 (✅ Fair)"
echo ""

sleep 1

echo "Vertex 12:"
echo "  → Event: BiasAuditComplete"
echo "  → Agent: did:example:security-engineer-dave"
echo "  → Data: {bias_detected: false, fairness_scores: {gender: 0.98, age: 0.97}}"
echo "  → Decision: APPROVED"
echo "  → Parent: bias-start-yz0..."
echo "  → Signature: [Dave's signature]"
echo "  → Hash: bias-approved-123..."
echo ""

sleep 2

echo "✅ Bias audit complete: Model is fair"
echo "✅ Ready for production deployment"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 5: Production Deployment (DevOps)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "DevOps Engineer Eve deploys to production:"
echo ""
echo "Vertex 13:"
echo "  → Event: DeploymentStarted"
echo "  → Agent: did:example:devops-engineer-eve"
echo "  → Data: {environment: 'production', replicas: 5}"
echo "  → Parent: bias-approved-123..."
echo "  → Hash: deploy-start-456..."
echo ""

sleep 1

echo "Vertex 14:"
echo "  → Event: DeploymentComplete"
echo "  → Agent: did:example:devops-engineer-eve"
echo "  → Data: {status: 'live', endpoint: 'fraud-api.prod:8080', health: 'healthy'}"
echo "  → Timestamp: 2025-12-26T18:00:00Z"
echo "  → Parent: deploy-start-456..."
echo "  → Signature: [Eve's signature]"
echo "  → Hash: deploy-complete-789..."
echo ""

sleep 2

echo "✅ Model deployed to production"
echo "✅ Serving fraud detection API"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 6: Merkle Proof & Dehydration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "Computing Merkle tree for complete pipeline..."
echo "  → 14 vertices (data prep → deployment)"
echo "  → 5 agents (Alice, Bob, Carol, Dave, Eve)"
echo "  → 10 hours total pipeline time"
echo ""

sleep 1

echo "✅ Merkle root: [134, 89, 203, 47, 156, ...]"
echo ""
echo "This root proves:"
echo "  • Complete pipeline from raw data → production"
echo "  • Every agent's contribution"
echo "  • All QA and security checks passed"
echo "  • Timestamps for every step"
echo "  • Model is auditable and traceable"
echo ""

sleep 2

echo "Dehydrating to LoamSpine for permanent record:"
cat << 'YAML'
dehydration_request:
  session: session-ml-fraud-detection-v2
  commit_what: full_pipeline
  include:
    - Data prep (vertices 1-4)
    - Training logs (vertices 5-8)
    - QA validation (vertices 9-10)
    - Bias audit (vertices 11-12)
    - Deployment (vertices 13-14)
    - Model checkpoint (NestGate reference)
  merkle_root: [134, 89, 203, 47, 156, ...]
  signatures:
    - data_engineer: did:example:data-engineer-alice
    - ml_engineer: did:example:ml-engineer-bob
    - qa_engineer: did:example:qa-engineer-carol
    - security: did:example:security-engineer-dave
    - devops: did:example:devops-engineer-eve
  compliance: SOC2_TYPE2_COMPLIANT
YAML

sleep 3

echo ""
echo "✅ Pipeline committed to LoamSpine"
echo "  → Commit ID: loam-commit-fraud-model-v2-production"
echo "  → All 5 signatures included"
echo "  → Complete provenance chain"
echo "  → Model now traceable in production"
echo ""

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 7: Later Provenance Query (Regulatory Audit)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

sleep 1

echo "3 months later, regulator asks:"
echo "'This model flagged a customer as fraud. Can you explain why?'"
echo ""

sleep 1

echo "Query to SweetGrass (provenance primal):"
cat << 'YAML'
provenance_query:
  model: fraud-detection-v2-production
  question: "Complete training provenance"
YAML

sleep 2

echo ""
echo "SweetGrass provides complete answer:"
cat << 'YAML'
model_provenance:
  model_id: fraud-detection-v2-production
  deployed: 2025-12-26T18:00:00Z
  
  data_lineage:
    source: production-db transactions-2024
    records: 10_000_000 → 9_943_000 (cleaned)
    prepared_by: did:example:data-engineer-alice
    features: 47 engineered features
    verification: Data prep signed and verified
    
  training_details:
    framework: XGBoost gradient boosting
    compute: ToadStool (8× NVIDIA A100)
    duration: 10 hours
    trained_by: did:example:ml-engineer-bob
    final_accuracy: 96%
    model_checkpoint: nestgate://checkpoint-xyz
    
  quality_assurance:
    validated_by: did:example:qa-engineer-carol
    test_set: 1M holdout samples
    metrics: {precision: 0.95, recall: 0.94, f1: 0.945}
    result: PASSED
    
  fairness_audit:
    audited_by: did:example:security-engineer-dave
    bias_check: PASSED
    parity_scores: {gender: 0.98, age: 0.97, geo: 0.95}
    approved: true
    
  deployment:
    deployed_by: did:example:devops-engineer-eve
    environment: production
    timestamp: 2025-12-26T18:00:00Z
    
  cryptographic_proof:
    merkle_root: [134, 89, 203, 47, 156, ...]
    all_signatures: verified
    chain_intact: true
    
  compliance: SOC2_TYPE2_COMPLIANT
YAML

sleep 3

echo ""
echo "✅ Complete answer with full provenance"
echo "✅ Regulator can verify every claim"
echo "✅ All signatures and timestamps proven"
echo "✅ Model is fully explainable"
echo ""

sleep 3

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 What rhizoCrypt Enabled"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Multi-Agent Coordination:"
echo "   5 agents across 3 orgs, all contributions tracked"
echo ""
echo "✅ Complete Pipeline Provenance:"
echo "   From raw data → production model with every step"
echo ""
echo "✅ Cryptographic Attribution:"
echo "   Every agent signed their work (via BearDog)"
echo ""
echo "✅ Quality & Fairness Verified:"
echo "   QA and security checks part of permanent record"
echo ""
echo "✅ Regulatory Compliance:"
echo "   Can answer any audit question with proof"
echo ""
echo "✅ Model Explainability:"
echo "   Complete training provenance for every prediction"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Key Insight:"
echo ""
echo "ML pipelines are complex, multi-agent workflows."
echo "rhizoCrypt captures every step, every decision, every agent."
echo "When regulators ask 'How did this model make this decision?'"
echo "you can provide complete, cryptographically-proven provenance."
echo ""
echo "This is the future of responsible AI."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Demo complete! ML pipelines with full provenance."
echo ""

