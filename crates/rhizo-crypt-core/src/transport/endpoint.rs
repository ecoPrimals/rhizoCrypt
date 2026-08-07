// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! [`TransportEndpoint`] — ecosystem canonical wire type for transport injection.

use crate::safe_env::SafeEnv;

/// Structured transport endpoint — wire-compatible with the ecosystem standard.
///
/// ```json
/// { "transport": "uds", "path": "/run/membrane/beardog.sock" }
/// { "transport": "tcp", "host": "192.168.1.144", "port": 7700 }
/// { "transport": "mesh_relay", "peer_id": "strand-gate", "capability": "security" }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {
    /// Unix Domain Socket.
    #[serde(rename = "uds")]
    Uds {
        /// Filesystem path to the socket.
        path: String,
    },
    /// TCP connection.
    #[serde(rename = "tcp")]
    Tcp {
        /// Host address.
        host: String,
        /// TCP port number.
        port: u16,
    },
    /// Mesh relay via Songbird.
    #[serde(rename = "mesh_relay")]
    MeshRelay {
        /// Mesh peer identifier.
        peer_id: String,
        /// Capability being resolved.
        capability: String,
    },
}

impl TransportEndpoint {
    /// Construct a TCP endpoint.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Returns `(host, port)` if this is a TCP endpoint.
    #[must_use]
    pub fn tcp_addr(&self) -> Option<(&str, u16)> {
        match self {
            Self::Tcp {
                host,
                port,
            } => Some((host, *port)),
            _ => None,
        }
    }

    /// Construct a UDS endpoint.
    #[must_use]
    pub fn uds(path: impl Into<String>) -> Self {
        Self::Uds {
            path: path.into(),
        }
    }

    /// Platform-default endpoint for a primal (G66).
    ///
    /// On Unix, returns UDS at the ecosystem socket path.
    /// On non-Unix, returns TCP localhost on `default_port`.
    #[must_use]
    pub fn platform_default(primal_name: &str, default_port: u16) -> Self {
        super::socket_path_for_primal(primal_name).map_or_else(
            || Self::tcp("127.0.0.1", default_port),
            |path| Self::Uds {
                path: path.to_string_lossy().into_owned(),
            },
        )
    }

    /// Read the transport endpoint from the environment, falling back to
    /// [`platform_default`](Self::platform_default) (G66).
    ///
    /// Checks `TRANSPORT_ENDPOINT` (JSON) and `{primal_env_prefix}_ADDRESS`
    /// (address string) before falling back.
    #[must_use]
    pub fn from_env_or_default(
        primal_name: &str,
        primal_env_prefix: &str,
        default_port: u16,
    ) -> Self {
        if let Some(json) = SafeEnv::get_optional("TRANSPORT_ENDPOINT")
            && let Ok(ep) = serde_json::from_str::<Self>(&json)
        {
            return ep;
        }

        let addr_key = format!("{primal_env_prefix}_ADDRESS");
        if let Some(addr) = SafeEnv::get_optional(&addr_key) {
            return Self::parse_address(&addr);
        }

        Self::platform_default(primal_name, default_port)
    }

    /// Returns `true` if this endpoint is local (UDS or TCP localhost).
    ///
    /// Used by G63 local-trust to decide whether `SO_PEERCRED` is available.
    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            Self::Uds {
                ..
            } => true,
            Self::Tcp {
                host,
                ..
            } => host == "127.0.0.1" || host == "::1" || host == "localhost",
            Self::MeshRelay {
                ..
            } => false,
        }
    }

    /// Try to parse an address string into a `TransportEndpoint`.
    ///
    /// Returns `None` for strings that don't look like valid addresses.
    /// UDS paths must start with `/`, TCP addresses must be `host:port`.
    #[must_use]
    pub fn try_parse_address(s: &str) -> Option<Self> {
        if s.starts_with('/') || s.to_ascii_lowercase().ends_with(".sock") {
            return Some(Self::Uds {
                path: s.to_string(),
            });
        }
        if let Some((host, port_str)) = s.rsplit_once(':')
            && let Ok(port) = port_str.parse::<u16>()
            && !host.is_empty()
        {
            return Some(Self::tcp(host, port));
        }
        None
    }

    /// Parse an address string into a `TransportEndpoint`.
    ///
    /// Heuristic: if the string contains `/` or ends in `.sock`, treat as UDS path.
    /// Otherwise try `host:port` for TCP. Falls back to UDS for unrecognized formats.
    #[must_use]
    pub fn parse_address(s: &str) -> Self {
        if s.contains('/') || s.to_ascii_lowercase().ends_with(".sock") {
            return Self::Uds {
                path: s.to_string(),
            };
        }
        if let Some((host, port_str)) = s.rsplit_once(':')
            && let Ok(port) = port_str.parse::<u16>()
        {
            return Self::tcp(host, port);
        }
        Self::Uds {
            path: s.to_string(),
        }
    }
}

impl From<std::net::SocketAddr> for TransportEndpoint {
    fn from(addr: std::net::SocketAddr) -> Self {
        Self::Tcp {
            host: addr.ip().to_string(),
            port: addr.port(),
        }
    }
}

impl std::fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uds {
                path,
            } => write!(f, "unix://{path}"),
            Self::Tcp {
                host,
                port,
            } => write!(f, "tcp://{host}:{port}"),
            Self::MeshRelay {
                peer_id,
                capability,
            } => write!(f, "mesh://{peer_id}/{capability}"),
        }
    }
}
