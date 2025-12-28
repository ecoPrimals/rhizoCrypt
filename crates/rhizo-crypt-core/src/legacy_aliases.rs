//! Legacy type aliases for backward compatibility.
//!
//! **DEPRECATED**: These aliases exist for backward compatibility only.
//! Use the new types from `types_ecosystem` directly.
//!
//! ## Migration Guide
//!
//! | Old (Deprecated) | New (Recommended) |
//! |------------------|-------------------|
//! | `ToadStoolConfig` | `types_ecosystem::compute::ComputeProviderConfig` |
//! | `ToadStoolClient` | `types_ecosystem::compute::ComputeProviderClient` |
//! | `SweetGrassConfig` | `types_ecosystem::provenance::ProvenanceProviderConfig` |
//! | `SweetGrassQueryable` | `types_ecosystem::provenance::ProvenanceQueryable` |

/// **DEPRECATED**: Use `ComputeProviderConfig` instead.
#[deprecated(since = "0.14.0", note = "Use `types_ecosystem::compute::ComputeProviderConfig` instead")]
pub type ToadStoolConfig = crate::types_ecosystem::compute::ComputeProviderConfig;

/// **DEPRECATED**: Use `ComputeProviderClient` instead.
#[deprecated(since = "0.14.0", note = "Use `types_ecosystem::compute::ComputeProviderClient` instead")]
pub type ToadStoolClient = crate::types_ecosystem::compute::ComputeProviderClient;

/// **DEPRECATED**: Use `ProvenanceProviderConfig` instead.
#[deprecated(since = "0.14.0", note = "Use `types_ecosystem::provenance::ProvenanceProviderConfig` instead")]
pub type SweetGrassConfig = crate::types_ecosystem::provenance::ProvenanceProviderConfig;

/// **DEPRECATED**: Use `ProvenanceNotifier` instead.
#[deprecated(since = "0.14.0", note = "Use `types_ecosystem::provenance::ProvenanceNotifier` instead")]
pub type SweetGrassNotifier = crate::types_ecosystem::provenance::ProvenanceNotifier;

/// **DEPRECATED**: Use `ProvenanceQueryable` trait instead.
#[deprecated(since = "0.14.0", note = "Use `types_ecosystem::provenance::ProvenanceQueryable` trait instead")]
pub type SweetGrassQueryable = Box<dyn crate::types_ecosystem::provenance::ProvenanceQueryable>;

