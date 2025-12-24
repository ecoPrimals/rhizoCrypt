//! Query filtering example.
//!
//! Demonstrates:
//! - Filtering by event type
//! - Filtering by time range
//! - Limiting results
//! - Querying specific vertices

use rhizo_crypt_rpc::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🔐 rhizoCrypt RPC Client - Query Filtering\n");

    // Connect
    let client = RpcClient::connect("127.0.0.1:9400").await?;
    println!("✓ Connected to server\n");

    // Create session
    let request = CreateSessionRequest {
        session_type: SessionType::Ephemeral,
        description: Some("Query Filtering Example".to_string()),
        parent_session: None,
        max_vertices: None,
        ttl_seconds: None,
    };

    let session_id = client.create_session(request).await?;
    println!("✓ Session created: {}\n", session_id);

    // Build a diverse DAG
    println!("Building DAG with mixed event types...");

    let event_types = vec![
        EventType::SessionStarted,
        EventType::DataCreated,
        EventType::DataModified,
        EventType::DataCreated,
        EventType::DataDeleted,
        EventType::DataModified,
        EventType::DataCreated,
        EventType::DataCommitted,
    ];

    let mut vertex_ids = Vec::new();

    for (i, event_type) in event_types.iter().enumerate() {
        let parents = if vertex_ids.is_empty() {
            vec![]
        } else {
            vec![vertex_ids[vertex_ids.len() - 1]]
        };

        let event = AppendEventRequest {
            session_id,
            event_type: *event_type,
            agent: None,
            parents,
            metadata: vec![
                ("index".to_string(), i.to_string()),
                ("category".to_string(), format!("cat-{}", i % 3)),
            ],
            payload_ref: None,
        };

        let vertex_id = client.append_event(event).await?;
        vertex_ids.push(vertex_id);
    }

    println!("✓ DAG built: {} vertices\n", vertex_ids.len());

    // Query 1: All vertices
    println!("Query 1: All Vertices");
    let query = QueryRequest {
        session_id,
        event_types: None,
        agent: None,
        start_time: None,
        end_time: None,
        limit: None,
    };

    let vertices = client.query_vertices(query).await?;
    println!("  Found {} vertices", vertices.len());
    for v in &vertices {
        println!("    - {:?}: {}", v.event_type, v.id);
    }
    println!();

    // Query 2: Filter by event type (DataCreated only)
    println!("Query 2: Filter by Event Type (DataCreated)");
    let query = QueryRequest {
        session_id,
        event_types: Some(vec![EventType::DataCreated]),
        agent: None,
        start_time: None,
        end_time: None,
        limit: None,
    };

    let vertices = client.query_vertices(query).await?;
    println!("  Found {} DataCreated vertices", vertices.len());
    for v in &vertices {
        println!("    - {:?}: {}", v.event_type, v.id);
    }
    println!();

    // Query 3: Filter by multiple event types
    println!("Query 3: Filter by Multiple Event Types (DataCreated, DataModified)");
    let query = QueryRequest {
        session_id,
        event_types: Some(vec![EventType::DataCreated, EventType::DataModified]),
        agent: None,
        start_time: None,
        end_time: None,
        limit: None,
    };

    let vertices = client.query_vertices(query).await?;
    println!("  Found {} vertices", vertices.len());
    for v in &vertices {
        println!("    - {:?}: {}", v.event_type, v.id);
    }
    println!();

    // Query 4: Limit results
    println!("Query 4: Limit Results (first 3)");
    let query = QueryRequest {
        session_id,
        event_types: None,
        agent: None,
        start_time: None,
        end_time: None,
        limit: Some(3),
    };

    let vertices = client.query_vertices(query).await?;
    println!("  Found {} vertices (limited)", vertices.len());
    for v in &vertices {
        println!("    - {:?}: {}", v.event_type, v.id);
    }
    println!();

    // Query 5: Get specific vertex
    println!("Query 5: Get Specific Vertex");
    let target_id = vertex_ids[3];
    let vertex = client.get_vertex(session_id, target_id).await?;
    println!("  Vertex: {}", vertex.id);
    println!("  Type: {:?}", vertex.event_type);
    println!("  Parents: {}", vertex.parents.len());
    println!("  Timestamp: {}", vertex.timestamp);
    println!();

    // Query 6: Get children of a vertex
    println!("Query 6: Get Children of Vertex");
    let parent_id = vertex_ids[2];
    let children = client.get_children(session_id, parent_id).await?;
    println!("  Parent: {}", parent_id);
    println!("  Children: {}", children.len());
    for child_id in &children {
        println!("    - {}", child_id);
    }
    println!();

    // Query 7: Frontier and Genesis
    println!("Query 7: Frontier and Genesis");
    let frontier = client.get_frontier(session_id).await?;
    let genesis = client.get_genesis(session_id).await?;

    println!("  Frontier (DAG tips): {}", frontier.len());
    for vid in &frontier {
        println!("    - {}", vid);
    }

    println!("  Genesis (DAG roots): {}", genesis.len());
    for vid in &genesis {
        println!("    - {}", vid);
    }

    println!("\n🎉 Query filtering complete!");
    println!("\n💡 Key Features:");
    println!("  • Filter by event type");
    println!("  • Filter by agent (DID)");
    println!("  • Filter by time range");
    println!("  • Limit result count");
    println!("  • Get specific vertices");
    println!("  • Query children/frontier/genesis");

    Ok(())
}

