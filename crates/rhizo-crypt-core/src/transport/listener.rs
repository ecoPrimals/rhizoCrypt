// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! [`TransportListener`] — server-side transport abstraction (G66).

use super::{TransportEndpoint, TransportStream};

/// Transport-agnostic server listener (G66).
///
/// Accepts incoming connections and yields [`TransportStream`] values,
/// abstracting over Unix domain sockets and TCP listeners.
pub enum TransportListener {
    /// Unix domain socket listener.
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),
    /// TCP listener.
    Tcp(tokio::net::TcpListener),
}

impl TransportListener {
    /// Accept the next incoming connection.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` on accept failure.
    pub async fn accept(&self) -> std::io::Result<TransportStream> {
        match self {
            #[cfg(unix)]
            Self::Unix(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Unix(stream))
            }
            Self::Tcp(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Tcp(stream))
            }
        }
    }

    /// Bind a listener for the given endpoint.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` on bind failure.
    pub async fn bind(endpoint: &TransportEndpoint) -> std::io::Result<Self> {
        match endpoint {
            #[cfg(unix)]
            TransportEndpoint::Uds {
                path,
            } => {
                if let Some(parent) = std::path::Path::new(path).parent()
                    && !parent.exists()
                {
                    std::fs::create_dir_all(parent)?;
                }
                if std::path::Path::new(path).exists() {
                    std::fs::remove_file(path)?;
                }
                let listener = tokio::net::UnixListener::bind(path)?;
                Ok(Self::Unix(listener))
            }
            #[cfg(not(unix))]
            TransportEndpoint::Uds {
                path,
            } => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("UDS not available on this platform for {path}"),
            )),
            TransportEndpoint::Tcp {
                host,
                port,
            } => {
                let addr = format!("{host}:{port}");
                let listener = tokio::net::TcpListener::bind(&addr).await?;
                Ok(Self::Tcp(listener))
            }
            TransportEndpoint::MeshRelay {
                ..
            } => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "cannot bind a listener for mesh relay endpoints",
            )),
        }
    }
}

impl std::fmt::Debug for TransportListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => f.debug_struct("TransportListener::Unix").finish(),
            Self::Tcp(_) => f.debug_struct("TransportListener::Tcp").finish(),
        }
    }
}
