//! RhizoCrypt error types.

use thiserror::Error;

/// Errors specific to RhizoCrypt.
#[derive(Debug, Error)]
pub enum RhizoCryptError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),
    
    // TODO: Add RhizoCrypt-specific errors
    
    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
