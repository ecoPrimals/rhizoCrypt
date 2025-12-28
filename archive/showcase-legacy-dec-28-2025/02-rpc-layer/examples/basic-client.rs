//! Basic RPC client example.
//!
//! Demonstrates:
//! - Connecting to rhizoCrypt RPC server
//! - Creating a session
//! - Appending events
//! - Querying the DAG

use rhizo_crypt_rpc::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔐 rhizoCrypt RPC Client - Basic Example\n");

    // Connect to server
    println!("Connecting to rhizoCrypt RPC server at 127.0.0.1:9400...");
    let client = RpcClient::connect("127.0.0.1:9400").await?;
    println!("✓ Connected!\n");

    // Create a session
    println!("Creating ephemeral session...");
    let request = CreateSessionRequest {
        session_type: SessionType::Ephemeral,
        description: Some("Basic RPC Example".to_string()),
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };

    let session_id = client.create_session(request).await?;
    println!("✓ Session created: {}\n", session_id);

    // Append an event
    println!("Appending event to session...");
    let event = AppendEventRequest {
        session_id,
        event_type: EventType::DataCreated,
        agent: None,
        parents: vec![],
        metadata: vec![
            ("action".to_string(), "create".to_string()),
            ("resource".to_string(), "document-1".to_string()),
        ],
        payload_ref: None,
    };

    let vertex_id = client.append_event(event).await?;
    println!("✓ Event appended: {}\n", vertex_id);

    // Append another event
    println!("Appending second event...");
    let event2 = AppendEventRequest {
        session_id,
        event_type: EventType::DataModified,
        agent: None,
        parents: vec![vertex_id],
        metadata: vec![
            ("action".to_string(), "modify".to_string()),
            ("resource".to_string(), "document-1".to_string()),
        ],
        payload_ref: None,
    };

    let vertex_id2 = client.append_event(event2).await?;
    println!("✓ Event appended: {}\n", vertex_id2);

    // Query the DAG
    println!("Querying DAG...");

    // Get frontier
    let frontier = client.get_frontier(session_id).await?;
    println!("  Frontier: {} vertices", frontier.len());
    for vid in &frontier {
        println!("    - {}", vid);
    }

    // Get genesis
    let genesis = client.get_genesis(session_id).await?;
    println!("  Genesis: {} vertices", genesis.len());
    for vid in &genesis {
        println!("    - {}", vid);
    }

    // Get session info
    println!("\nGetting session info...");
    let info = client.get_session(session_id).await?;
    println!("  Session ID: {}", info.id);
    println!("  Type: {:?}", info.session_type);
    println!("  State: {:?}", info.state);
    println!("  Vertices: {}", info.vertex_count);

    // Get Merkle root
    println!("\nComputing Merkle root...");
    let root = client.get_merkle_root(session_id).await?;
    println!("  Merkle root: {}", root);

    // Health check
    println!("\nChecking server health...");
    let health = client.health().await?;
    println!("  Healthy: {}", health.healthy);
    println!("  State: {}", health.state);
    println!("  Active sessions: {}", health.active_sessions);
    println!("  Total vertices: {}", health.total_vertices);
    println!("  Uptime: {}s", health.uptime_seconds);

    // Metrics
    println!("\nGetting service metrics...");
    let metrics = client.metrics().await?;
    println!("  Sessions created: {}", metrics.sessions_created);
    println!("  Vertices appended: {}", metrics.vertices_appended);
    println!("  Queries executed: {}", metrics.queries_executed);

    println!("\n🎉 Example complete!");

    Ok(())
}

