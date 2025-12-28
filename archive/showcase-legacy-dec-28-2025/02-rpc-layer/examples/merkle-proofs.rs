//! Merkle proof example.
//!
//! Demonstrates:
//! - Computing Merkle root
//! - Generating inclusion proofs
//! - Verifying proofs
//! - Proof size analysis

use rhizo_crypt_rpc::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🔐 rhizoCrypt RPC Client - Merkle Proofs\n");

    // Connect
    let client = RpcClient::connect("127.0.0.1:9400").await?;
    println!("✓ Connected to server\n");

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::Ephemeral,
        description: Some("Merkle Proof Example".to_string()),
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };

    let session_id = client.create_session(request).await?;
    println!("✓ Session created: {}\n", session_id);

    // Build a DAG with multiple vertices
    println!("Building DAG with 10 vertices...");
    let mut vertex_ids = Vec::new();

    for i in 0..10 {
        let parents = if vertex_ids.is_empty() {
            vec![]
        } else {
            vec![vertex_ids[vertex_ids.len() - 1]]
        };

        let event = AppendEventRequest {
            session_id,
            event_type: EventType::DataCreated,
            agent: None,
            parents,
            metadata: vec![("index".to_string(), i.to_string())],
            payload_ref: None,
        };

        let vertex_id = client.append_event(event).await?;
        vertex_ids.push(vertex_id);
    }

    println!("✓ DAG built: {} vertices\n", vertex_ids.len());

    // Compute Merkle root
    println!("Computing Merkle root...");
    let root = client.get_merkle_root(session_id).await?;
    println!("  Merkle root: {}\n", root);

    // Generate proof for middle vertex
    let target_vertex = vertex_ids[5];
    println!("Generating proof for vertex {}...", target_vertex);
    let proof = client.get_merkle_proof(session_id, target_vertex).await?;

    println!("  ✓ Proof generated");
    println!("  Vertex ID: {}", proof.vertex_id);
    println!("  Proof path length: {}", proof.path.len());
    println!("  Proof size: ~{} bytes\n", proof.path.len() * 32);

    // Verify proof
    println!("Verifying proof...");
    let valid = client.verify_proof(root.clone(), proof.clone()).await?;

    if valid {
        println!("  ✅ Proof VALID - Vertex is in the DAG!");
    } else {
        println!("  ❌ Proof INVALID");
    }

    // Analyze proof efficiency
    println!("\nProof Efficiency Analysis:");
    let dag_size = vertex_ids.len();
    let proof_size = proof.path.len();
    let full_dag_size = dag_size * 200; // ~200 bytes per vertex
    let proof_bytes = proof_size * 32; // 32 bytes per hash

    println!("  DAG size: {} vertices (~{} bytes)", dag_size, full_dag_size);
    println!("  Proof size: {} hashes ({} bytes)", proof_size, proof_bytes);
    let savings = 100.0 * (1.0 - proof_bytes as f64 / full_dag_size as f64);
    println!("  Space savings: {:.1}%", savings);

    // Generate proofs for all vertices
    println!("\nGenerating proofs for all vertices...");
    for (i, vertex_id) in vertex_ids.iter().enumerate() {
        let proof = client.get_merkle_proof(session_id, *vertex_id).await?;
        println!("  Vertex {}: proof path length = {}", i, proof.path.len());
    }

    println!("\n🎉 Merkle proof example complete!");
    println!("\n💡 Key Insights:");
    println!("  • Proof size is O(log n) - very compact!");
    println!("  • {:.0}% space savings vs full DAG", savings);
    println!("  • Proofs enable selective disclosure");
    println!("  • Cryptographic integrity guaranteed");

    Ok(())
}

