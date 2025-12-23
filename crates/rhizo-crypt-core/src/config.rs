//! RhizoCrypt configuration.

use serde::{Deserialize, Serialize};
use sourdough_core::config::CommonConfig;

/// Configuration for RhizoCrypt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RhizoCryptConfig {
    /// Common configuration.
    #[serde(flatten)]
    pub common: CommonConfig,
    
    // TODO: Add RhizoCrypt-specific configuration
}

impl Default for RhizoCryptConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                name: "RhizoCrypt".to_string(),
                ..CommonConfig::default()
            },
        }
    }
}
