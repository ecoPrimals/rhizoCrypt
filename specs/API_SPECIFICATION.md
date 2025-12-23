# RhizoCrypt — API Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 22, 2025

---

## 1. Overview

RhizoCrypt exposes two API interfaces:
- **gRPC API** — High-performance, strongly-typed, streaming support
- **REST API** — HTTP/JSON for compatibility and tooling

Both APIs are exposed through Songbird's Universal Port Authority (UPA).

---

## 2. gRPC API

### 2.1 Service Definition

```protobuf
syntax = "proto3";

package rhizocrypt.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";

// The RhizoCrypt service
service RhizoCrypt {
    // ==================== Session Management ====================
    
    // Create a new session
    rpc CreateSession(CreateSessionRequest) returns (CreateSessionResponse);
    
    // Get session details
    rpc GetSession(GetSessionRequest) returns (GetSessionResponse);
    
    // List sessions with filtering
    rpc ListSessions(ListSessionsRequest) returns (ListSessionsResponse);
    
    // Pause a session
    rpc PauseSession(PauseSessionRequest) returns (PauseSessionResponse);
    
    // Resume a paused session
    rpc ResumeSession(ResumeSessionRequest) returns (ResumeSessionResponse);
    
    // Resolve a session (triggers dehydration)
    rpc ResolveSession(ResolveSessionRequest) returns (ResolveSessionResponse);
    
    // Discard a session without committing
    rpc DiscardSession(DiscardSessionRequest) returns (DiscardSessionResponse);
    
    // ==================== Event Operations ====================
    
    // Append a single event
    rpc AppendEvent(AppendEventRequest) returns (AppendEventResponse);
    
    // Append multiple events in a batch
    rpc AppendEventBatch(AppendEventBatchRequest) returns (AppendEventBatchResponse);
    
    // Stream events to a session
    rpc StreamEvents(stream AppendEventRequest) returns (stream AppendEventResponse);
    
    // ==================== Query Operations ====================
    
    // Get a vertex by ID
    rpc GetVertex(GetVertexRequest) returns (GetVertexResponse);
    
    // Get multiple vertices
    rpc GetVertices(GetVerticesRequest) returns (GetVerticesResponse);
    
    // Query vertices with filters
    rpc QueryVertices(QueryVerticesRequest) returns (stream Vertex);
    
    // Get session frontier (tip vertices)
    rpc GetFrontier(GetFrontierRequest) returns (GetFrontierResponse);
    
    // Traverse DAG from a starting vertex
    rpc TraverseDAG(TraverseDAGRequest) returns (stream Vertex);
    
    // ==================== Slice Operations ====================
    
    // Checkout a slice from LoamSpine
    rpc CheckoutSlice(CheckoutSliceRequest) returns (CheckoutSliceResponse);
    
    // Get slice status
    rpc GetSlice(GetSliceRequest) returns (GetSliceResponse);
    
    // List slices in a session
    rpc ListSlices(ListSlicesRequest) returns (ListSlicesResponse);
    
    // Manually resolve a slice
    rpc ResolveSlice(ResolveSliceRequest) returns (ResolveSliceResponse);
    
    // ==================== Merkle Operations ====================
    
    // Compute Merkle root for a session
    rpc ComputeMerkleRoot(ComputeMerkleRootRequest) returns (ComputeMerkleRootResponse);
    
    // Generate Merkle proof for a vertex
    rpc GenerateMerkleProof(GenerateMerkleProofRequest) returns (GenerateMerkleProofResponse);
    
    // Verify a Merkle proof
    rpc VerifyMerkleProof(VerifyMerkleProofRequest) returns (VerifyMerkleProofResponse);
    
    // ==================== Dehydration ====================
    
    // Manually trigger dehydration
    rpc Dehydrate(DehydrateRequest) returns (DehydrateResponse);
    
    // Get dehydration status
    rpc GetDehydrationStatus(GetDehydrationStatusRequest) returns (GetDehydrationStatusResponse);
    
    // ==================== Health & Metrics ====================
    
    // Health check
    rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
    
    // Get metrics
    rpc GetMetrics(GetMetricsRequest) returns (GetMetricsResponse);
}
```

### 2.2 Message Definitions

```protobuf
// ==================== Common Types ====================

message VertexId {
    bytes hash = 1;  // 32-byte Blake3 hash
}

message SessionId {
    string uuid = 1;  // UUID v7 string
}

message SliceId {
    string uuid = 1;
}

message Did {
    string value = 1;  // did:key:z6Mk...
}

message PayloadRef {
    bytes hash = 1;
    uint64 size = 2;
    optional string mime_type = 3;
}

message Signature {
    bytes value = 1;
}

// ==================== Session Messages ====================

message CreateSessionRequest {
    SessionType session_type = 1;
    optional string name = 2;
    SessionConfig config = 3;
    Did owner = 4;
}

message CreateSessionResponse {
    SessionId session_id = 1;
    Session session = 2;
}

message SessionType {
    oneof type {
        GamingSession gaming = 1;
        ExperimentSession experiment = 2;
        CollaborationSession collaboration = 3;
        GeneralSession general = 4;
        CustomSession custom = 5;
    }
}

message GamingSession {
    string game_id = 1;
}

message ExperimentSession {
    string protocol_id = 1;
}

message CollaborationSession {
    string workspace_id = 1;
}

message GeneralSession {}

message CustomSession {
    string domain = 1;
}

message SessionConfig {
    google.protobuf.Duration max_duration = 1;
    uint64 max_vertices = 2;
    uint64 max_payload_bytes = 3;
    bool require_all_signatures = 4;
    repeated string signature_required_events = 5;
    bool auto_dehydrate = 6;
    DehydrationConfig dehydration = 7;
    StorageBackend storage_backend = 8;
}

enum StorageBackend {
    STORAGE_BACKEND_MEMORY = 0;
    STORAGE_BACKEND_ROCKSDB = 1;
    STORAGE_BACKEND_LMDB = 2;
}

message DehydrationConfig {
    bool include_vertices = 1;
    bool include_payloads = 2;
    repeated VertexId generate_proofs_for = 3;
    repeated Did required_attestations = 4;
}

message Session {
    SessionId id = 1;
    optional string name = 2;
    SessionType session_type = 3;
    SessionConfig config = 4;
    uint64 created_at = 5;
    SessionState state = 6;
    uint64 vertex_count = 7;
    uint64 frontier_size = 8;
    uint64 slice_count = 9;
    repeated Did agents = 10;
}

message SessionState {
    oneof state {
        ActiveState active = 1;
        PausedState paused = 2;
        ResolvingState resolving = 3;
        CommittedState committed = 4;
        DiscardedState discarded = 5;
        ExpiredState expired = 6;
    }
}

message ActiveState {}
message PausedState { string reason = 1; }
message ResolvingState { uint64 started_at = 1; }
message CommittedState { LoamCommitRef loam_ref = 1; uint64 committed_at = 2; }
message DiscardedState { string reason = 1; uint64 discarded_at = 2; }
message ExpiredState { uint64 expired_at = 1; }

message LoamCommitRef {
    string spine_id = 1;
    bytes entry_hash = 2;
    uint64 index = 3;
}

// ==================== Event Messages ====================

message AppendEventRequest {
    SessionId session_id = 1;
    EventType event_type = 2;
    optional bytes payload = 3;
    repeated VertexId parents = 4;  // Auto-detected if empty
    optional Did agent = 5;
    optional Signature signature = 6;
    map<string, string> metadata = 7;
}

message AppendEventResponse {
    VertexId vertex_id = 1;
    uint64 timestamp = 2;
}

message AppendEventBatchRequest {
    SessionId session_id = 1;
    repeated EventBatchItem events = 2;
}

message EventBatchItem {
    EventType event_type = 1;
    optional bytes payload = 2;
    repeated VertexId parents = 3;
    optional Did agent = 4;
    optional Signature signature = 5;
    map<string, string> metadata = 6;
}

message AppendEventBatchResponse {
    repeated VertexId vertex_ids = 1;
}

message EventType {
    string domain = 1;
    string name = 2;
    map<string, string> attributes = 3;
}

// ==================== Vertex Messages ====================

message Vertex {
    VertexId id = 1;
    repeated VertexId parents = 2;
    uint64 timestamp = 3;
    optional Did agent = 4;
    optional Signature signature = 5;
    EventType event_type = 6;
    optional PayloadRef payload = 7;
    map<string, string> metadata = 8;
}

message GetVertexRequest {
    SessionId session_id = 1;
    VertexId vertex_id = 2;
}

message GetVertexResponse {
    optional Vertex vertex = 1;
}

message GetVerticesRequest {
    SessionId session_id = 1;
    repeated VertexId vertex_ids = 2;
}

message GetVerticesResponse {
    repeated Vertex vertices = 1;
}

message QueryVerticesRequest {
    SessionId session_id = 1;
    optional VertexFilter filter = 2;
    optional uint32 limit = 3;
    optional string cursor = 4;
}

message VertexFilter {
    repeated string event_types = 1;
    repeated Did agents = 2;
    optional uint64 timestamp_start = 3;
    optional uint64 timestamp_end = 4;
    repeated PayloadRef payload_refs = 5;
}

// ==================== Slice Messages ====================

message CheckoutSliceRequest {
    SessionId session_id = 1;
    string spine_id = 2;
    bytes entry_hash = 3;
    SliceMode mode = 4;
    ResolutionRoute resolution_route = 5;
    SliceConstraints constraints = 6;
    Did requester = 7;
}

message CheckoutSliceResponse {
    Slice slice = 1;
    VertexId checkout_vertex = 2;
}

message SliceMode {
    oneof mode {
        CopyMode copy = 1;
        LoanMode loan = 2;
        ConsignmentMode consignment = 3;
        EscrowMode escrow = 4;
        WaypointMode waypoint = 5;
        TransferMode transfer = 6;
    }
}

message CopyMode {
    bool allow_recopy = 1;
}

message LoanMode {
    LoanTerms terms = 1;
    bool allow_subloan = 2;
}

message LoanTerms {
    optional google.protobuf.Duration duration = 1;
    optional google.protobuf.Duration grace_period = 2;
    bool auto_return = 3;
}

message ConsignmentMode {
    Did consignee = 1;
    repeated ResolutionTrigger resolution_triggers = 2;
}

message EscrowMode {
    repeated Did parties = 1;
    uint32 required_confirmations = 2;
}

message WaypointMode {
    string waypoint_spine = 1;
}

message TransferMode {
    Did new_owner = 1;
}

message ResolutionRoute {
    oneof route {
        ReturnToOrigin return_to_origin = 1;
        CommitToOrigin commit_to_origin = 2;
        RouteToSpine route_to_spine = 3;
        WaypointReturn waypoint_return = 4;
        ConditionalRoute conditional = 5;
    }
}

message ReturnToOrigin {}
message CommitToOrigin { bool include_summary = 1; }
message RouteToSpine { string target_spine = 1; }
message WaypointReturn { string waypoint_spine = 1; }
message ConditionalRoute { repeated ConditionalRouteEntry conditions = 1; ResolutionRoute default = 2; }

message ConditionalRouteEntry {
    ResolutionCondition condition = 1;
    ResolutionRoute route = 2;
}

message ResolutionCondition {
    oneof condition {
        bool session_success = 1;
        bool session_rollback = 2;
        bool session_timeout = 3;
        string event_occurred = 4;
        string external_trigger = 5;
        bool all_parties_confirmed = 6;
        bool loan_expired = 7;
        bool owner_recall = 8;
    }
}

message ResolutionTrigger {
    oneof trigger {
        string event_type = 1;
        google.protobuf.Duration timeout = 2;
        string external_trigger = 3;
    }
}

message SliceConstraints {
    optional google.protobuf.Duration max_duration = 1;
    bool allow_reslice = 2;
    optional uint32 max_reslice_depth = 3;
    repeated string forbidden_operations = 4;
}

message Slice {
    SliceId id = 1;
    SliceOrigin origin = 2;
    Did holder = 3;
    Did owner = 4;
    SliceMode mode = 5;
    ResolutionRoute resolution_route = 6;
    uint64 checked_out_at = 7;
    optional uint64 expires_at = 8;
    SliceConstraints constraints = 9;
    SliceState state = 10;
    SessionId session_id = 11;
    VertexId checkout_vertex = 12;
}

message SliceOrigin {
    string spine_id = 1;
    bytes entry_hash = 2;
    uint64 entry_index = 3;
    optional string certificate_id = 4;
    Did owner = 5;
}

message SliceState {
    oneof state {
        SliceActiveState active = 1;
        SliceAnchoredState anchored = 2;
        SliceResolvingState resolving = 3;
        SliceResolvedState resolved = 4;
    }
}

message SliceActiveState { SessionId session_id = 1; }
message SliceAnchoredState { string waypoint_spine = 1; bytes anchor_entry = 2; }
message SliceResolvingState { uint64 started_at = 1; }
message SliceResolvedState { ResolutionOutcome outcome = 1; uint64 resolved_at = 2; }

message ResolutionOutcome {
    oneof outcome {
        bool returned_unchanged = 1;
        bytes committed_entry = 2;
        TransferredOutcome transferred = 3;
        AnchoredOutcome anchored = 4;
        bool consumed = 5;
    }
}

message TransferredOutcome {
    string new_spine = 1;
    bytes new_entry = 2;
    Did new_owner = 3;
}

message AnchoredOutcome {
    string waypoint_spine = 1;
    bytes waypoint_entry = 2;
}

// ==================== Merkle Messages ====================

message ComputeMerkleRootRequest {
    SessionId session_id = 1;
}

message ComputeMerkleRootResponse {
    MerkleRoot merkle_root = 1;
    uint64 vertex_count = 2;
}

message MerkleRoot {
    bytes hash = 1;
}

message GenerateMerkleProofRequest {
    SessionId session_id = 1;
    VertexId vertex_id = 2;
}

message GenerateMerkleProofResponse {
    MerkleProof proof = 1;
}

message MerkleProof {
    VertexId vertex_id = 1;
    uint32 position = 2;
    uint32 total_vertices = 3;
    repeated MerkleProofSibling siblings = 4;
    MerkleRoot root = 5;
}

message MerkleProofSibling {
    Direction direction = 1;
    bytes hash = 2;
}

enum Direction {
    DIRECTION_LEFT = 0;
    DIRECTION_RIGHT = 1;
}

message VerifyMerkleProofRequest {
    MerkleProof proof = 1;
    Vertex vertex = 2;
}

message VerifyMerkleProofResponse {
    bool valid = 1;
}
```

---

## 3. REST API

### 3.1 OpenAPI Specification

```yaml
openapi: 3.0.3
info:
  title: RhizoCrypt API
  description: Ephemeral DAG Engine for ecoPrimals
  version: 0.2.0
  license:
    name: AGPL-3.0
    url: https://www.gnu.org/licenses/agpl-3.0.en.html

servers:
  - url: /api/v1/rhizocrypt
    description: Local RhizoCrypt instance

paths:
  # ==================== Sessions ====================
  
  /sessions:
    post:
      summary: Create a new session
      operationId: createSession
      tags: [Sessions]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateSessionRequest'
      responses:
        '201':
          description: Session created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Session'
        '400':
          $ref: '#/components/responses/BadRequest'
    
    get:
      summary: List sessions
      operationId: listSessions
      tags: [Sessions]
      parameters:
        - name: state
          in: query
          schema:
            type: string
            enum: [active, paused, resolving, committed, discarded, expired]
        - name: session_type
          in: query
          schema:
            type: string
        - name: limit
          in: query
          schema:
            type: integer
            default: 50
        - name: cursor
          in: query
          schema:
            type: string
      responses:
        '200':
          description: Session list
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SessionList'

  /sessions/{session_id}:
    get:
      summary: Get session details
      operationId: getSession
      tags: [Sessions]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      responses:
        '200':
          description: Session details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Session'
        '404':
          $ref: '#/components/responses/NotFound'
    
    delete:
      summary: Discard session
      operationId: discardSession
      tags: [Sessions]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                reason:
                  type: string
      responses:
        '200':
          description: Session discarded
        '404':
          $ref: '#/components/responses/NotFound'

  /sessions/{session_id}/pause:
    post:
      summary: Pause session
      operationId: pauseSession
      tags: [Sessions]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                reason:
                  type: string
      responses:
        '200':
          description: Session paused

  /sessions/{session_id}/resume:
    post:
      summary: Resume paused session
      operationId: resumeSession
      tags: [Sessions]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      responses:
        '200':
          description: Session resumed

  /sessions/{session_id}/resolve:
    post:
      summary: Resolve session (trigger dehydration)
      operationId: resolveSession
      tags: [Sessions]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ResolveRequest'
      responses:
        '200':
          description: Session resolved
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ResolveResponse'

  # ==================== Events ====================
  
  /sessions/{session_id}/events:
    post:
      summary: Append event to session
      operationId: appendEvent
      tags: [Events]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/AppendEventRequest'
      responses:
        '201':
          description: Event appended
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AppendEventResponse'
    
    get:
      summary: Query events in session
      operationId: queryEvents
      tags: [Events]
      parameters:
        - $ref: '#/components/parameters/SessionId'
        - name: event_type
          in: query
          schema:
            type: string
        - name: agent
          in: query
          schema:
            type: string
        - name: since
          in: query
          schema:
            type: integer
            format: int64
        - name: until
          in: query
          schema:
            type: integer
            format: int64
        - name: limit
          in: query
          schema:
            type: integer
            default: 100
      responses:
        '200':
          description: Vertex list
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/VertexList'

  /sessions/{session_id}/events/batch:
    post:
      summary: Append batch of events
      operationId: appendEventBatch
      tags: [Events]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/AppendEventBatchRequest'
      responses:
        '201':
          description: Events appended
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AppendEventBatchResponse'

  # ==================== Vertices ====================
  
  /sessions/{session_id}/vertices/{vertex_id}:
    get:
      summary: Get vertex by ID
      operationId: getVertex
      tags: [Vertices]
      parameters:
        - $ref: '#/components/parameters/SessionId'
        - $ref: '#/components/parameters/VertexId'
      responses:
        '200':
          description: Vertex details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Vertex'
        '404':
          $ref: '#/components/responses/NotFound'

  /sessions/{session_id}/frontier:
    get:
      summary: Get session frontier (tip vertices)
      operationId: getFrontier
      tags: [Vertices]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      responses:
        '200':
          description: Frontier vertices
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/VertexList'

  # ==================== Slices ====================
  
  /sessions/{session_id}/slices:
    post:
      summary: Checkout a slice from LoamSpine
      operationId: checkoutSlice
      tags: [Slices]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CheckoutSliceRequest'
      responses:
        '201':
          description: Slice checked out
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/CheckoutSliceResponse'
    
    get:
      summary: List slices in session
      operationId: listSlices
      tags: [Slices]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      responses:
        '200':
          description: Slice list
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SliceList'

  /sessions/{session_id}/slices/{slice_id}:
    get:
      summary: Get slice details
      operationId: getSlice
      tags: [Slices]
      parameters:
        - $ref: '#/components/parameters/SessionId'
        - $ref: '#/components/parameters/SliceId'
      responses:
        '200':
          description: Slice details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Slice'

  /sessions/{session_id}/slices/{slice_id}/resolve:
    post:
      summary: Manually resolve a slice
      operationId: resolveSlice
      tags: [Slices]
      parameters:
        - $ref: '#/components/parameters/SessionId'
        - $ref: '#/components/parameters/SliceId'
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ResolveSliceRequest'
      responses:
        '200':
          description: Slice resolved

  # ==================== Merkle ====================
  
  /sessions/{session_id}/merkle:
    get:
      summary: Compute Merkle root
      operationId: computeMerkleRoot
      tags: [Merkle]
      parameters:
        - $ref: '#/components/parameters/SessionId'
      responses:
        '200':
          description: Merkle root
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MerkleRootResponse'

  /sessions/{session_id}/merkle/proof/{vertex_id}:
    get:
      summary: Generate Merkle proof for vertex
      operationId: generateMerkleProof
      tags: [Merkle]
      parameters:
        - $ref: '#/components/parameters/SessionId'
        - $ref: '#/components/parameters/VertexId'
      responses:
        '200':
          description: Merkle proof
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MerkleProof'

  /merkle/verify:
    post:
      summary: Verify a Merkle proof
      operationId: verifyMerkleProof
      tags: [Merkle]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/VerifyMerkleProofRequest'
      responses:
        '200':
          description: Verification result
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/VerifyMerkleProofResponse'

  # ==================== Health ====================
  
  /health:
    get:
      summary: Health check
      operationId: healthCheck
      tags: [Health]
      responses:
        '200':
          description: Service healthy
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HealthResponse'

  /metrics:
    get:
      summary: Get metrics
      operationId: getMetrics
      tags: [Health]
      responses:
        '200':
          description: Prometheus metrics
          content:
            text/plain:
              schema:
                type: string

components:
  parameters:
    SessionId:
      name: session_id
      in: path
      required: true
      schema:
        type: string
        format: uuid
    
    VertexId:
      name: vertex_id
      in: path
      required: true
      schema:
        type: string
        pattern: '^[a-f0-9]{64}$'
    
    SliceId:
      name: slice_id
      in: path
      required: true
      schema:
        type: string
        format: uuid

  responses:
    BadRequest:
      description: Invalid request
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    
    NotFound:
      description: Resource not found
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'

  schemas:
    # ... schema definitions (abbreviated for length)
    Error:
      type: object
      properties:
        code:
          type: string
        message:
          type: string
        details:
          type: object
```

---

## 4. WebSocket API

For real-time event streaming:

```
WebSocket: /ws/sessions/{session_id}/events

Messages (server → client):
- vertex.appended: New vertex in session
- session.state_changed: Session state transition
- slice.resolved: Slice resolution complete

Messages (client → server):
- subscribe: Subscribe to event types
- unsubscribe: Unsubscribe from event types
```

---

## 5. Authentication

All API calls require BearDog authentication:

```
Authorization: Bearer <beardog-token>
X-BearDog-DID: did:key:z6Mk...
```

---

## 6. Rate Limiting

| Endpoint Category | Rate Limit |
|-------------------|------------|
| Session management | 100 req/min |
| Event append | 10,000 req/min |
| Query operations | 1,000 req/min |
| Slice operations | 100 req/min |

---

## 7. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md) — Primal integrations

---

*RhizoCrypt: The memory that knows when to forget.*

