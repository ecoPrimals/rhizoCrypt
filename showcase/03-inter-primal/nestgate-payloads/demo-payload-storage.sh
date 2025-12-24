#!/bin/bash
#
# 🔐 rhizoCrypt NestGate Payload Demo
#
# Demonstrates content-addressed payload storage via NestGate.
#

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
PURPLE='\033[0;35m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }

echo -e "${PURPLE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════╗
║          🔐 rhizoCrypt NestGate Payload Demo                   ║
║                                                                ║
║  Demonstrates: Payload Storage • Content Addressing            ║
╚═══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Get the path to rhizoCrypt BEFORE changing directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RHIZO_PATH="$(cd "$SCRIPT_DIR/../../.." && pwd)"

DEMO_DIR="/tmp/rhizocrypt-payload-demo"
rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

cat > Cargo.toml << EOF
[package]
name = "payload-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
rhizo-crypt-core = { path = "$RHIZO_PATH/crates/rhizo-crypt-core", features = ["test-utils"] }
tokio = { version = "1.0", features = ["full"] }
hex = "0.4"
blake3 = "1.5"
bytes = "1.0"
EOF

mkdir -p src

cat > src/main.rs << 'EOF'
//! NestGate Payload Demo
//!
//! Shows content-addressed payload storage and retrieval.

use rhizo_crypt_core::{
    EventType, PrimalLifecycle, RhizoCrypt, RhizoCryptConfig,
    SessionBuilder, SessionType, VertexBuilder,
    types::{ContentHash, PayloadRef},
    integration::{NestGateClient, MockNestGateClient},
};
use bytes::Bytes;

// Simulate a payload (e.g., an image or file)
struct SimulatedPayload {
    data: Bytes,
    content_hash: ContentHash,
}

impl SimulatedPayload {
    fn new(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self {
            data: Bytes::copy_from_slice(data),
            content_hash: hash.into(),
        }
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 rhizoCrypt NestGate Payload Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Initialize rhizoCrypt
    let config = RhizoCryptConfig::default();
    let mut primal = RhizoCrypt::new(config);
    primal.start().await?;

    // Create a MockNestGateClient (simulates NestGate)
    let nestgate = MockNestGateClient::new();

    // Create a session
    println!("📦 Creating session...\n");
    let session = SessionBuilder::new(SessionType::Experiment {
        protocol_id: "payload-demo".to_string(),
    })
    .with_name("Payload Storage Demo")
    .build();
    let session_id = primal.create_session(session).await?;
    println!("   Session: {}\n", session_id);

    // Simulate creating some payloads
    println!("📁 Creating payloads...\n");

    let image_payload = SimulatedPayload::new(b"[Simulated 5MB image data...]");
    let audio_payload = SimulatedPayload::new(b"[Simulated 10MB audio data...]");
    let model_payload = SimulatedPayload::new(b"[Simulated 50MB model weights...]");

    println!("   Image: {} bytes, hash: {}", 
        image_payload.size(), 
        hex::encode(&image_payload.content_hash[..8]));
    println!("   Audio: {} bytes, hash: {}", 
        audio_payload.size(), 
        hex::encode(&audio_payload.content_hash[..8]));
    println!("   Model: {} bytes, hash: {}\n", 
        model_payload.size(), 
        hex::encode(&model_payload.content_hash[..8]));

    // Store payloads in NestGate (simulated)
    println!("💾 Storing payloads in NestGate (simulated)...\n");

    let image_ref = nestgate.put_payload(image_payload.data.clone()).await?;
    println!("   ✓ Stored image: {}", image_ref);

    let audio_ref = nestgate.put_payload(audio_payload.data.clone()).await?;
    println!("   ✓ Stored audio: {}", audio_ref);

    let model_ref = nestgate.put_payload(model_payload.data.clone()).await?;
    println!("   ✓ Stored model: {}\n", model_ref);

    // Create vertices that reference payloads
    println!("🔗 Linking payloads to DAG vertices...\n");

    let genesis = VertexBuilder::new(EventType::SessionStart).build();
    let v1 = primal.append_vertex(session_id, genesis).await?;
    println!("   v1: Session started");

    let v2 = VertexBuilder::new(EventType::DataCreate { schema: Some("image".to_string()) })
        .with_parent(v1)
        .with_payload(image_ref.clone())
        .with_metadata("mime", "image/png".to_string())
        .build();
    let v2_id = primal.append_vertex(session_id, v2).await?;
    println!("   v2: Image uploaded → {}", v2_id);

    let v3 = VertexBuilder::new(EventType::DataCreate { schema: Some("audio".to_string()) })
        .with_parent(v2_id)
        .with_payload(audio_ref.clone())
        .with_metadata("mime", "audio/wav".to_string())
        .build();
    let v3_id = primal.append_vertex(session_id, v3).await?;
    println!("   v3: Audio uploaded → {}", v3_id);

    let v4 = VertexBuilder::new(EventType::DataCreate { schema: Some("model".to_string()) })
        .with_parent(v3_id)
        .with_payload(model_ref.clone())
        .with_metadata("mime", "application/octet-stream".to_string())
        .build();
    let v4_id = primal.append_vertex(session_id, v4).await?;
    println!("   v4: Model uploaded → {}\n", v4_id);

    // Demonstrate retrieval
    println!("📥 Retrieving payloads from NestGate (simulated)...\n");

    if let Some(retrieved) = nestgate.get_payload(&image_ref).await? {
        println!("   ✓ Retrieved image: {} bytes", retrieved.len());
    }

    if let Some(retrieved) = nestgate.get_payload(&audio_ref).await? {
        println!("   ✓ Retrieved audio: {} bytes", retrieved.len());
    }

    if let Some(retrieved) = nestgate.get_payload(&model_ref).await? {
        println!("   ✓ Retrieved model: {} bytes\n", retrieved.len());
    }

    // Show the separation of concerns
    println!("📊 Storage Separation:\n");
    println!("   ┌─────────────────────────────────────────────────┐");
    println!("   │                rhizoCrypt (DAG)                 │");
    println!("   │  ┌─────────────────────────────────────────────┐│");
    println!("   │  │ v1 → v2 → v3 → v4                           ││");
    println!("   │  │      ↓    ↓    ↓                            ││");
    println!("   │  │    ref₁  ref₂  ref₃   ← PayloadRefs only!   ││");
    println!("   │  └─────────────────────────────────────────────┘│");
    println!("   │  Size: ~1KB (metadata + refs)                   │");
    println!("   └─────────────────────────────────────────────────┘");
    println!();
    println!("   ┌─────────────────────────────────────────────────┐");
    println!("   │               NestGate (Content)                │");
    println!("   │  ┌─────────────────────────────────────────────┐│");
    println!("   │  │ ref₁ → [5MB image]                          ││");
    println!("   │  │ ref₂ → [10MB audio]                         ││");
    println!("   │  │ ref₃ → [50MB model]                         ││");
    println!("   │  └─────────────────────────────────────────────┘│");
    println!("   │  Size: 65MB (actual content)                    │");
    println!("   └─────────────────────────────────────────────────┘\n");

    // Cleanup
    primal.discard_session(session_id).await?;
    primal.stop().await?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Payload Demo completed!");
    println!();
    println!("Key Takeaways:");
    println!("  • rhizoCrypt stores structure, NestGate stores content");
    println!("  • PayloadRefs are content-addressed (Blake3)");
    println!("  • Same content = same hash (deduplication)");
    println!("  • Vertices reference payloads, don't embed them");
    println!("  • Efficient: 65MB content, ~1KB in DAG");
    println!();

    Ok(())
}
EOF

log "Building demo..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

log "Running payload storage demo..."
echo ""
cargo run --release 2>&1

rm -rf "$DEMO_DIR"
success "Demo complete!"
