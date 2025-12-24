//! Error handling example.
//!
//! Demonstrates:
//! - Handling RPC errors
//! - Retry logic
//! - Graceful degradation
//! - Error classification

use rhizo_crypt_rpc::*;
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🔐 rhizoCrypt RPC Client - Error Handling\n");

    // Connect
    let client = RpcClient::connect("127.0.0.1:9400").await?;
    println!("✓ Connected to server\n");

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::Ephemeral,
        description: Some("Error Handling Example".to_string()),
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };

    let session_id = client.create_session(request).await?;
    println!("✓ Session created: {}\n", session_id);

    // Example 1: Handle session not found
    println!("Example 1: Session Not Found");
    let fake_session_id = SessionId::now(); // Random session ID

    match client.get_session(fake_session_id).await {
        Ok(info) => println!("  Unexpected success: {:?}", info),
        Err(RpcError::SessionNotFound(msg)) => {
            println!("  ✓ Handled SessionNotFound: {}", msg);
        }
        Err(e) => println!("  Other error: {:?}", e),
    }
    println!();

    // Example 2: Handle vertex not found
    println!("Example 2: Vertex Not Found");
    let fake_vertex_id = VertexId::ZERO; // Non-existent vertex

    match client.get_vertex(session_id, fake_vertex_id).await {
        Ok(vertex) => println!("  Unexpected success: {:?}", vertex),
        Err(RpcError::VertexNotFound(msg)) => {
            println!("  ✓ Handled VertexNotFound: {}", msg);
        }
        Err(e) => println!("  Other error: {:?}", e),
    }
    println!();

    // Example 3: Retry logic
    println!("Example 3: Retry Logic");
    let max_retries = 3;
    let mut attempt = 0;

    let result = loop {
        attempt += 1;
        println!("  Attempt {}/{}", attempt, max_retries);

        let event = AppendEventRequest {
            session_id,
            event_type: EventType::DataCreated,
            agent: None,
            parents: vec![],
            metadata: vec![("attempt".to_string(), attempt.to_string())],
            payload_ref: None,
        };

        match client.append_event(event).await {
            Ok(vertex_id) => {
                println!("  ✓ Success on attempt {}", attempt);
                break Ok(vertex_id);
            }
            Err(e) if attempt < max_retries => {
                println!("  ⚠ Failed: {:?}, retrying...", e);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => {
                println!("  ✗ Failed after {} attempts", max_retries);
                break Err(e);
            }
        }
    };

    match result {
        Ok(id) => println!("  Final result: {}", id),
        Err(e) => println!("  Final error: {:?}", e),
    }
    println!();

    // Example 4: Graceful degradation
    println!("Example 4: Graceful Degradation");

    // Try to get metrics, fall back to basic info
    let metrics_result = client.metrics().await;

    match metrics_result {
        Ok(metrics) => {
            println!("  ✓ Got metrics:");
            println!("    Sessions created: {}", metrics.sessions_created);
            println!("    Vertices appended: {}", metrics.vertices_appended);
        }
        Err(e) => {
            println!("  ⚠ Metrics unavailable: {:?}", e);
            println!("  Falling back to health check...");

            match client.health().await {
                Ok(health) => {
                    println!("  ✓ Got basic health info:");
                    println!("    Healthy: {}", health.healthy);
                    println!("    Active sessions: {}", health.active_sessions);
                }
                Err(e) => {
                    println!("  ✗ Health check also failed: {:?}", e);
                }
            }
        }
    }
    println!();

    // Example 5: Error classification
    println!("Example 5: Error Classification");

    let errors = vec![
        ("SessionNotFound", RpcError::SessionNotFound("test".to_string())),
        ("VertexNotFound", RpcError::VertexNotFound("test".to_string())),
        ("SliceNotFound", RpcError::SliceNotFound("test".to_string())),
        ("Core", RpcError::Core("test".to_string())),
    ];

    for (name, error) in errors {
        let is_retryable = matches!(error, RpcError::Core(_));
        let is_not_found = matches!(
            error,
            RpcError::SessionNotFound(_) | RpcError::VertexNotFound(_) | RpcError::SliceNotFound(_)
        );

        println!("  {}: retryable={}, not_found={}", name, is_retryable, is_not_found);
    }

    println!("\n🎉 Error handling examples complete!");
    println!("\n💡 Best Practices:");
    println!("  • Match on specific error types");
    println!("  • Implement retry logic for transient errors");
    println!("  • Use graceful degradation");
    println!("  • Classify errors (retryable vs fatal)");
    println!("  • Log errors for debugging");
    println!("  • Provide user-friendly error messages");

    Ok(())
}

