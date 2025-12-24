//! Performance Benchmarks for rhizoCrypt Core
//!
//! Measures critical path performance for:
//! - Vertex creation and content addressing
//! - DAG operations (store, query, frontier)
//! - Merkle tree computation
//!
//! Run with: `cargo bench -p rhizo-crypt-core`

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::let_underscore_must_use)]
#![allow(unused_must_use)]
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rhizo_crypt_core::{
    event::EventType,
    merkle::{MerkleRoot, MerkleTreeBuilder},
    session::SessionBuilder,
    store::InMemoryDagStore,
    types::{SessionId, VertexId},
    vertex::VertexBuilder,
    DagStore, SessionType,
};
use tokio::runtime::Runtime;

// ============================================================================
// Vertex Creation Benchmarks
// ============================================================================

fn bench_vertex_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_creation");

    // Benchmark basic vertex creation
    group.bench_function("basic_vertex", |b| {
        b.iter(|| {
            let vertex = VertexBuilder::new(EventType::SessionStart).build();
            black_box(vertex)
        });
    });

    // Benchmark vertex with metadata
    group.bench_function("vertex_with_metadata_5", |b| {
        b.iter(|| {
            let vertex = VertexBuilder::new(EventType::DataCreate {
                schema: None,
            })
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2")
            .with_metadata("key3", "value3")
            .with_metadata("key4", "value4")
            .with_metadata("key5", "value5")
            .build();
            black_box(vertex)
        });
    });

    // Benchmark vertex with parents
    group.bench_function("vertex_with_parents_3", |b| {
        let parents: Vec<VertexId> = (0u8..3).map(|i| VertexId::from_bytes(&[i; 32])).collect();

        b.iter(|| {
            let mut builder = VertexBuilder::new(EventType::DataCreate {
                schema: None,
            });
            for parent in &parents {
                builder = builder.with_parent(*parent);
            }
            let vertex = builder.build();
            black_box(vertex)
        });
    });

    // Benchmark session builder
    group.bench_function("session_builder", |b| {
        b.iter(|| {
            let session = SessionBuilder::new(SessionType::General)
                .with_name("bench-session")
                .with_max_vertices(10000)
                .build();
            black_box(session)
        });
    });

    group.finish();
}

// ============================================================================
// Content Addressing Benchmarks
// ============================================================================

fn bench_content_addressing(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_addressing");

    // Benchmark vertex ID computation
    group.bench_function("compute_vertex_id", |b| {
        let vertex = VertexBuilder::new(EventType::SessionStart).build();
        b.iter(|| {
            let id = vertex.compute_id();
            black_box(id)
        });
    });

    // Benchmark VertexId from bytes (Blake3 hash)
    for size in [32u32, 256, 1024, 4096] {
        group.throughput(Throughput::Bytes(u64::from(size)));
        group.bench_with_input(BenchmarkId::new("blake3_hash", size), &size, |b, &size| {
            let data: Vec<u8> = (0..size).map(|i| (i & 0xFF) as u8).collect();
            b.iter(|| {
                let id = VertexId::from_bytes(black_box(&data));
                black_box(id)
            });
        });
    }

    group.finish();
}

// ============================================================================
// Merkle Tree Benchmarks
// ============================================================================

fn bench_merkle_tree(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_tree");

    // Benchmark Merkle root computation for different sizes
    for count in [10u32, 100, 1000] {
        let vertices: Vec<_> = (0..count)
            .map(|i| {
                VertexBuilder::new(EventType::DataCreate {
                    schema: Some(format!("schema-{i}")),
                })
                .with_metadata("index", i.to_string())
                .build()
            })
            .collect();

        group.throughput(Throughput::Elements(u64::from(count)));
        group.bench_with_input(
            BenchmarkId::new("compute_root", count),
            &vertices,
            |b, vertices| {
                b.iter(|| {
                    let root = MerkleRoot::compute(black_box(vertices));
                    black_box(root)
                });
            },
        );
    }

    // Benchmark proof generation
    for count in [10u32, 100, 1000] {
        let vertices: Vec<_> = (0..count)
            .map(|i| {
                VertexBuilder::new(EventType::DataCreate {
                    schema: None,
                })
                .with_metadata("index", i.to_string())
                .build()
            })
            .collect();

        let mut builder = MerkleTreeBuilder::new();
        builder.add_vertices(vertices.clone());

        group.bench_with_input(
            BenchmarkId::new("generate_proof", count),
            &builder,
            |b, builder| {
                b.iter(|| {
                    // Generate proof for middle element
                    let proof = builder.generate_proof(black_box((count / 2) as usize));
                    black_box(proof)
                });
            },
        );
    }

    // Benchmark proof verification
    group.bench_function("verify_proof", |b| {
        let vertices: Vec<_> = (0..100)
            .map(|i| {
                VertexBuilder::new(EventType::DataCreate {
                    schema: None,
                })
                .with_metadata("index", i.to_string())
                .build()
            })
            .collect();

        let mut builder = MerkleTreeBuilder::new();
        builder.add_vertices(vertices.clone());
        let proof =
            builder.generate_proof(50).expect("proof generation should succeed for valid index");

        b.iter(|| {
            let valid = proof.verify(black_box(&vertices[50]));
            black_box(valid)
        });
    });

    group.finish();
}

// ============================================================================
// DAG Store Benchmarks
// ============================================================================

fn bench_dag_store(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime creation should succeed");
    let mut group = c.benchmark_group("dag_store");

    // Benchmark vertex put
    group.bench_function("put_vertex", |b| {
        b.iter_custom(|iters| {
            let store = InMemoryDagStore::new();
            let session_id = SessionId::now();

            rt.block_on(async {
                let start = std::time::Instant::now();

                for i in 0..iters {
                    let vertex = VertexBuilder::new(EventType::DataCreate {
                        schema: Some(format!("schema-{i}")),
                    })
                    .build();
                    let _ = store.put_vertex(session_id, vertex).await;
                }

                start.elapsed()
            })
        });
    });

    // Benchmark vertex get
    group.bench_function("get_vertex", |b| {
        let store = InMemoryDagStore::new();
        let session_id = SessionId::now();

        let vertex_id = rt.block_on(async {
            // Populate with some vertices
            for i in 0..100 {
                let vertex = VertexBuilder::new(EventType::DataCreate {
                    schema: Some(format!("schema-{i}")),
                })
                .build();
                let _ = store.put_vertex(session_id, vertex).await;
            }

            // Get a vertex ID to lookup
            store
                .get_frontier(session_id)
                .await
                .ok()
                .and_then(|f| f.first().copied())
                .expect("frontier should have at least one vertex")
        });

        b.iter(|| {
            rt.block_on(async {
                let vertex = store.get_vertex(session_id, black_box(vertex_id)).await;
                black_box(vertex)
            });
        });
    });

    // Benchmark frontier query
    group.bench_function("get_frontier", |b| {
        let store = InMemoryDagStore::new();
        let session_id = SessionId::now();

        rt.block_on(async {
            // Create a DAG with multiple frontier vertices
            let genesis = VertexBuilder::new(EventType::SessionStart).build();
            let mut genesis_clone = genesis.clone();
            let genesis_id = genesis_clone.id();
            let _ = store.put_vertex(session_id, genesis).await;

            // Add several children to create a wider frontier
            for i in 0..10 {
                let child = VertexBuilder::new(EventType::DataCreate {
                    schema: None,
                })
                .with_parent(genesis_id)
                .with_metadata("branch", i.to_string())
                .build();
                let _ = store.put_vertex(session_id, child).await;
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let frontier = store.get_frontier(black_box(session_id)).await;
                black_box(frontier)
            });
        });
    });

    // Benchmark genesis query
    group.bench_function("get_genesis", |b| {
        let store = InMemoryDagStore::new();
        let session_id = SessionId::now();

        rt.block_on(async {
            // Create some genesis vertices
            for _ in 0..5 {
                let vertex = VertexBuilder::new(EventType::SessionStart).build();
                let _ = store.put_vertex(session_id, vertex).await;
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let genesis = store.get_genesis(black_box(session_id)).await;
                black_box(genesis)
            });
        });
    });

    // Benchmark children query
    group.bench_function("get_children", |b| {
        let store = InMemoryDagStore::new();
        let session_id = SessionId::now();

        let parent_id = rt.block_on(async {
            // Create parent
            let parent = VertexBuilder::new(EventType::SessionStart).build();
            let mut parent_clone = parent.clone();
            let parent_id = parent_clone.id();
            let _ = store.put_vertex(session_id, parent).await;

            // Add children
            for i in 0..20 {
                let child = VertexBuilder::new(EventType::DataCreate {
                    schema: None,
                })
                .with_parent(parent_id)
                .with_metadata("child", i.to_string())
                .build();
                let _ = store.put_vertex(session_id, child).await;
            }

            parent_id
        });

        b.iter(|| {
            rt.block_on(async {
                let children = store.get_children(session_id, black_box(parent_id)).await;
                black_box(children)
            });
        });
    });

    // Benchmark vertex count
    group.bench_function("count_vertices", |b| {
        let store = InMemoryDagStore::new();
        let session_id = SessionId::now();

        rt.block_on(async {
            for i in 0..1000 {
                let vertex = VertexBuilder::new(EventType::DataCreate {
                    schema: None,
                })
                .with_metadata("index", i.to_string())
                .build();
                let _ = store.put_vertex(session_id, vertex).await;
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let count = store.count_vertices(black_box(session_id)).await;
                black_box(count)
            });
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_vertex_creation,
    bench_content_addressing,
    bench_merkle_tree,
    bench_dag_store,
);

criterion_main!(benches);
