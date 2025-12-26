#!/usr/bin/env bash
# Demo: Complete ML Pipeline - All Primals Working Together
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🤖 Complete ML Pipeline: All Primals${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

cd "$(dirname "$0")/../.."

cat > /tmp/ml_pipeline.rs << 'EOF'
use rhizo_crypt_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Complete ML Pipeline: rhizoCrypt Orchestrates All");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;
    
    println!("🎯 Scenario: Image Classification Model Training");
    println!("   Dataset: ImageNet subset");
    println!("   Model: ResNet-50");
    println!("   Team: 5 researchers, 2 ML engineers");
    println!("");
    
    let session = SessionBuilder::new(SessionType::General)
        .with_name("resnet-training-v2")
        .with_owner(Did::new("did:key:ml-team-lead"))
        .build();
    let session_id = primal.create_session(session).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 1: Data Preparation & Storage (NestGate)");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Upload dataset
    println!("📦 Uploading Dataset to NestGate");
    let dataset = b"[ImageNet data: 50,000 images, 224x224, RGB]".repeat(100);
    let dataset_payload = PayloadRef::from_bytes(&dataset);
    let dataset_hash = blake3::hash(&dataset);
    
    let v1 = VertexBuilder::new(EventType::DataCreate { schema: None })
        .with_agent(Did::new("did:key:data-engineer"))
        .with_payload(dataset_payload)
        .with_metadata("stage", "data_upload")
        .with_metadata("dataset", "imagenet-subset")
        .with_metadata("images", "50000")
        .with_metadata("size_gb", "12.5")
        .with_metadata("storage", "nestgate")
        .build();
    let v1_id = primal.append_vertex(session_id, v1).await?;
    
    println!("   ✓ Dataset uploaded");
    println!("     Hash: {}", dataset_hash);
    println!("     Size: 12.5 GB");
    println!("     Location: NestGate (content-addressed)");
    println!("     Vertex: {}", v1_id);
    println!("");
    
    // Data validation
    println!("✅ Validating Dataset");
    let v2 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:data-validator"))
        .with_parent(v1_id)
        .with_metadata("stage", "validation")
        .with_metadata("status", "passed")
        .with_metadata("corrupt_images", "0")
        .with_metadata("format_errors", "0")
        .build();
    let v2_id = primal.append_vertex(session_id, v2).await?;
    println!("   ✓ Dataset validated");
    println!("     Status: Passed");
    println!("     Corrupt images: 0");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 2: Training Request & Execution (ToadStool)");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Training request
    println!("🎓 Requesting GPU Training from ToadStool");
    let v3 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:ml-engineer"))
        .with_parent(v2_id)
        .with_metadata("stage", "training_request")
        .with_metadata("model", "resnet-50")
        .with_metadata("gpus", "8")
        .with_metadata("framework", "pytorch")
        .with_metadata("epochs", "100")
        .with_metadata("batch_size", "256")
        .build();
    let v3_id = primal.append_vertex(session_id, v3).await?;
    println!("   ✓ Training requested");
    println!("     Model: ResNet-50");
    println!("     GPUs: 8× NVIDIA A100");
    println!("     Epochs: 100");
    println!("");
    
    // ToadStool training
    println!("⚙️  ToadStool Training (8 GPUs)...");
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    let model = b"[ResNet-50 weights: 25.6M parameters]".repeat(100);
    let model_payload = PayloadRef::from_bytes(&model);
    let model_hash = blake3::hash(&model);
    
    let v4 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:toadstool:coordinator"))
        .with_parent(v3_id)
        .with_payload(model_payload)
        .with_metadata("stage", "training_complete")
        .with_metadata("accuracy", "0.92")
        .with_metadata("loss", "0.08")
        .with_metadata("duration_hours", "6.2")
        .with_metadata("gpu_hours", "49.6")
        .with_metadata("cost_usd", "148.80")
        .build();
    let v4_id = primal.append_vertex(session_id, v4).await?;
    
    println!("   ✓ Training complete");
    println!("     Accuracy: 92%");
    println!("     Loss: 0.08");
    println!("     Duration: 6.2 hours");
    println!("     GPU hours: 49.6");
    println!("     Cost: $148.80");
    println!("     Model hash: {}", model_hash);
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 3: Model Storage & Signing (NestGate + BearDog)");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Store model in NestGate
    println!("💾 Storing Model in NestGate");
    let v5 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:ml-ops"))
        .with_parent(v4_id)
        .with_metadata("stage", "model_storage")
        .with_metadata("storage", "nestgate")
        .with_metadata("model_hash", &model_hash.to_string())
        .with_metadata("size_mb", "98.4")
        .build();
    let v5_id = primal.append_vertex(session_id, v5).await?;
    println!("   ✓ Model stored in NestGate");
    println!("     Hash: {}", model_hash);
    println!("     Size: 98.4 MB");
    println!("");
    
    // Sign model with BearDog
    println!("🔐 Signing Model with BearDog HSM");
    let signature = b"[Ed25519 signature over model hash]";
    
    let v6 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:beardog:hsm-prod-1"))
        .with_parent(v5_id)
        .with_metadata("stage", "model_signing")
        .with_metadata("signature", &hex::encode(signature))
        .with_metadata("algorithm", "Ed25519")
        .with_metadata("signer", "ml-team-release-key")
        .build();
    let v6_id = primal.append_vertex(session_id, v6).await?;
    println!("   ✓ Model signed");
    println!("     Algorithm: Ed25519");
    println!("     Signer: ml-team-release-key");
    println!("     Signature: {}...", hex::encode(&signature[0..16]));
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 4: Validation & Release Approval");
    println!("═══════════════════════════════════════════════════════\n");
    
    // Validation
    println!("✅ Validating Model Performance");
    let v7 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:qa-engineer"))
        .with_parent(v6_id)
        .with_metadata("stage", "validation")
        .with_metadata("test_accuracy", "0.91")
        .with_metadata("inference_ms", "45")
        .with_metadata("status", "passed")
        .build();
    let v7_id = primal.append_vertex(session_id, v7).await?;
    println!("   ✓ Validation passed");
    println!("     Test accuracy: 91%");
    println!("     Inference time: 45ms");
    println!("");
    
    // Release approval
    println!("📢 Release Approval");
    let v8 = VertexBuilder::new(EventType::DataUpdate { schema: None })
        .with_agent(Did::new("did:key:ml-team-lead"))
        .with_parent(v7_id)
        .with_metadata("stage", "release_approval")
        .with_metadata("version", "resnet-50-v2.0")
        .with_metadata("status", "approved")
        .with_metadata("deployment", "production")
        .build();
    let _v8_id = primal.append_vertex(session_id, v8).await?;
    println!("   ✓ Release approved");
    println!("     Version: resnet-50-v2.0");
    println!("     Deployment: Production");
    println!("");
    
    // Resolve session
    let resolution = primal.resolve_session(session_id, ResolutionOutcome::Commit).await?;
    
    println!("═══════════════════════════════════════════════════════");
    println!("  🎉 Complete ML Pipeline: Success!");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("📊 Full Workflow:");
    println!("");
    println!("   Data Engineer    → Upload dataset (NestGate)");
    println!("   Data Validator   → Validate dataset");
    println!("   ML Engineer      → Request training (ToadStool)");
    println!("   ToadStool        → Train model (8 GPUs)");
    println!("   ML Ops           → Store model (NestGate)");
    println!("   BearDog HSM      → Sign model");
    println!("   QA Engineer      → Validate model");
    println!("   Team Lead        → Approve release");
    println!("");
    
    println!("🔐 Provenance Proof:");
    println!("   • 8 vertices (8 agents)");
    println!("   • 4 primals (rhizoCrypt, NestGate, ToadStool, BearDog)");
    println!("   • Merkle root: {}", hex::encode(&resolution.merkle_root));
    println!("   • Single cryptographic proof of entire pipeline");
    println!("");
    
    println!("✨ What This Demonstrates:");
    println!("   • rhizoCrypt orchestrates, doesn't embed");
    println!("   • Each primal does what it does best");
    println!("   • Full provenance: dataset → training → model → deployment");
    println!("   • Multi-agent collaboration");
    println!("   • Cost, performance, and compliance tracking");
    println!("   • Zero-knowledge discovery (no hardcoded endpoints)");
    println!("");
    
    println!("═══════════════════════════════════════════════════════");
    println!("  This is the power of the ecoPrimals ecosystem!");
    println!("═══════════════════════════════════════════════════════\n");
    
    Ok(())
}
EOF

echo -e "${YELLOW}Compiling ML pipeline demo...${NC}"
rustc --edition 2021 /tmp/ml_pipeline.rs \
    -L target/release/deps \
    --extern rhizo_crypt_core=target/release/librhizo_crypt_core.rlib \
    --extern tokio=target/release/deps/libtokio-*.rlib \
    --extern blake3=target/release/deps/libblake3-*.rlib \
    --extern hex=target/release/deps/libhex-*.rlib \
    -o /tmp/ml_pipeline 2>&1 | grep -v "warning" || true

echo "Running ML pipeline demo..."
echo ""
/tmp/ml_pipeline

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Complete ML pipeline demo complete!${NC}"
echo ""
echo -e "${YELLOW}📚 What you learned:${NC}"
echo "  • rhizoCrypt orchestrates all primals"
echo "  • NestGate: Data and model storage"
echo "  • ToadStool: GPU training"
echo "  • BearDog: Model signing"
echo "  • Complete provenance: dataset → production"
echo ""
echo -e "${YELLOW}▶ Next demo:${NC} ./demo-document-workflow.sh"
echo ""

rm -f /tmp/ml_pipeline.rs /tmp/ml_pipeline

