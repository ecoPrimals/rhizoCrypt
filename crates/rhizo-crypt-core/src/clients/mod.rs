//! Live Primal Clients
//!
//! Real implementations for connecting to live primals in the ecosystem.
//!
//! ## Architecture
//!
//! - **Phase 1 Primals**: BearDog (identity), Songbird (discovery), NestGate (storage)
//! - **Phase 2 Siblings**: LoamSpine (permanence), ToadStool (compute), SweetGrass (provenance)
//!
//! ## Features
//!
//! - **Default mode**: Scaffolded clients that verify connectivity but don't
//!   perform actual RPC calls (for development/testing).
//! - **`live-clients` feature**: Enables actual tarpc/HTTP client connections
//!   to sibling primals.
//!
//! ## Discovery Pattern
//!
//! All clients use capability-based discovery via Songbird. No primal has
//! compile-time knowledge of other primals' addresses.

// Phase 1 primal clients
pub mod beardog;
pub mod nestgate;
pub mod songbird;

// Phase 2 sibling clients
pub mod loamspine;
pub mod sweetgrass;
pub mod toadstool;

// Live client modules (only with live-clients feature)
#[cfg(feature = "live-clients")]
pub mod beardog_http;
#[cfg(feature = "live-clients")]
pub mod loamspine_rpc;
#[cfg(feature = "live-clients")]
pub mod nestgate_http;
#[cfg(feature = "live-clients")]
pub mod songbird_rpc;
#[cfg(feature = "live-clients")]
pub mod toadstool_http;

// Phase 1 exports
pub use beardog::{BearDogClient, BearDogConfig};
pub use nestgate::{NestGateClient, NestGateConfig};
pub use songbird::{SongbirdClient, SongbirdConfig};

// Phase 2 exports
pub use loamspine::{LoamSpineClient, LoamSpineConfig};
pub use sweetgrass::{
    AgentContribution, ProvenanceChain, SessionAttribution, SweetGrassConfig, SweetGrassNotifier,
    SweetGrassQueryable, VertexQuery, VertexRef,
};
pub use toadstool::{ComputeEvent, TaskId, ToadStoolClient, ToadStoolConfig};

// Re-export types when feature is enabled
#[cfg(feature = "live-clients")]
pub use beardog_http::{BearDogHttpClient, BearDogHttpError};
#[cfg(feature = "live-clients")]
pub use loamspine_rpc::{
    LoamSpineRpc, LoamSpineRpcClient, RpcCommitSessionRequest, RpcCommitSessionResponse,
};
#[cfg(feature = "live-clients")]
pub use nestgate_http::{NestGateHttpClient, NestGateHttpError};
#[cfg(feature = "live-clients")]
pub use songbird_rpc::{
    RpcHealthStatus, RpcRegistrationResult, RpcServiceInfo, RpcServiceRegistration, RpcVersionInfo,
    SongbirdRpc, SongbirdRpcClient,
};
#[cfg(feature = "live-clients")]
pub use toadstool_http::{
    DeploymentResponse, DeploymentStatus, HealthStatus as ToadStoolHealthStatus, ResourceUsage,
    ToadStoolHttpClient, ToadStoolHttpError,
};
