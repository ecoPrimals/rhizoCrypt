//! Batch operations example.
//!
//! Demonstrates:
//! - Batch event appending
//! - Performance comparison (batch vs individual)
//! - Error handling

use rhizo_crypt_rpc::*;
use std::error::Error;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🔐 rhizoCrypt RPC Client - Batch Operations\n");

    // Connect
    let client = RpcClient::connect("127.0.0.1:9400").await?;
    println!("✓ Connected to server\n");

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::Ephemeral,
        description: Some("Batch Operations Example".to_string()),
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };

    let session_id = client.create_session(request).await?;
    println!("✓ Session created: {}\n", session_id);

    // Test 1: Individual appends
    println!("Test 1: Individual Appends (100 events)");
    let start = Instant::now();

    for i in 0..100 {
        let event = AppendEventRequest {
            session_id,
            event_type: EventType::DataCreated,
            agent: None,
            parents: vec![],
            metadata: vec![("index".to_string(), i.to_string())],
            payload_ref: None,
        };

        client.append_event(event).await?;
    }

    let individual_time = start.elapsed();
    println!("  ✓ Completed in {:?}", individual_time);
    println!("  Average: {:?} per event\n", individual_time / 100);

    // Create new session for batch test
    let request2 = CreateSessionRequest {
        session_type: SessionType::Ephemeral,
        description: Some("Batch Test Session".to_string()),
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };

    let session_id2 = client.create_session(request2).await?;

    // Test 2: Batch append
    println!("Test 2: Batch Append (100 events)");
    let start = Instant::now();

    let events: Vec<_> = (0..100)
        .map(|i| AppendEventRequest {
            session_id: session_id2,
            event_type: EventType::DataCreated,
            agent: None,
            parents: vec![],
            metadata: vec![("index".to_string(), i.to_string())],
            payload_ref: None,
        })
        .collect();

    let vertex_ids = client.append_batch(events).await?;

    let batch_time = start.elapsed();
    println!("  ✓ Completed in {:?}", batch_time);
    println!("  Average: {:?} per event", batch_time / 100);
    println!("  Returned {} vertex IDs\n", vertex_ids.len());

    // Compare
    println!("Performance Comparison:");
    println!("  Individual: {:?}", individual_time);
    println!("  Batch:      {:?}", batch_time);
    let speedup = individual_time.as_secs_f64() / batch_time.as_secs_f64();
    println!("  Speedup:    {:.2}x faster\n", speedup);

    // Verify both sessions
    let info1 = client.get_session(session_id).await?;
    let info2 = client.get_session(session_id2).await?;

    println!("Session 1 (individual): {} vertices", info1.vertex_count);
    println!("Session 2 (batch):      {} vertices", info2.vertex_count);

    println!("\n🎉 Batch operations complete!");
    println!("\n💡 Key Takeaway: Batch operations are ~{:.0}x faster!", speedup);

    Ok(())
}

