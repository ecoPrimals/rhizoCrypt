// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! BTSP wire limits, storage key geometry, and genetics-layer signal constants.

// ============================================================================
// STORAGE KEY GEOMETRY
// ============================================================================

/// Size of a `SessionId` in bytes (UUID v7 = 128 bits).
///
/// Source: RFC 9562 (UUID v7) — 128-bit timestamp-ordered identifier.
pub const SESSION_ID_BYTES: usize = 16;

/// Size of a `VertexId` in bytes (BLAKE3 hash = 256 bits).
///
/// Source: BLAKE3 specification — 256-bit output is the default digest size.
pub const VERTEX_ID_BYTES: usize = 32;

/// Separator byte between session and vertex in composite keys.
///
/// Chosen: ASCII colon (0x3A) — never appears in hex-encoded IDs,
/// enables visual separation in debug output.
pub const VERTEX_KEY_SEPARATOR: u8 = b':';

/// Total size of a composite `session:vertex` key.
///
/// Derivation: 16 (UUID v7) + 1 (separator) + 32 (BLAKE3) = 49 bytes.
pub const VERTEX_KEY_SIZE: usize = SESSION_ID_BYTES + 1 + VERTEX_ID_BYTES;

/// Estimated average bytes per stored vertex (for memory estimation).
///
/// Derivation: empirical measurement across spring experiment sessions.
/// Median vertex: 32B hash + 16B session + 64B event + 48B metadata +
/// 32B parents + 64B overhead ≈ 256 B. Validated: session 3 benchmarks.
pub const ESTIMATED_BYTES_PER_VERTEX: u64 = 256;

// ============================================================================
// COMPRESSION CONSTANTS
// ============================================================================

/// Maximum line length for newline-delimited JSON-RPC (16 MiB).
///
/// Derivation: matches BTSP `MAX_FRAME_SIZE` (16 MiB). Largest realistic
/// single JSON-RPC request is a batch dehydration summary (~1 MiB). 16 MiB
/// provides generous headroom while preventing unbounded memory allocation
/// from misbehaving or adversarial clients.
pub const MAX_JSONRPC_LINE_LENGTH: usize = 16 * 1024 * 1024;

/// Default compression threshold (1 KB).
///
/// Payloads smaller than this are not compressed.
pub const DEFAULT_COMPRESSION_THRESHOLD: usize = 1024;

// ============================================================================
// GENETICS-LAYER SIGNAL CONSTANTS (Eukaryotic Model)
// ============================================================================

/// Mito-beacon signal byte: shared/copyable relay access, mesh transport.
///
/// Part of the eukaryotic genetics model. The first byte of a 2-byte signal
/// prefix on UDS connections. `BearDog` owns the signal namespace.
/// `FAMILY_SEED` is mito-beacon material (legacy naming).
pub const MITO_BEACON_SIGNAL: u8 = 0xEC;

/// Mito-beacon extended signal byte: relay-mesh variant.
pub const MITO_BEACON_EXTENDED: u8 = 0xED;

/// Nuclear lineage signal byte: per-user permissions, tiered access.
///
/// BearDog-spawned, non-fungible. Wave 115+ evolution — primals should
/// recognize but not yet act on this signal.
pub const NUCLEAR_LINEAGE_SIGNAL: u8 = 0xEE;

/// Check whether a byte is a recognized genetics-layer signal.
#[inline]
#[must_use]
pub const fn is_genetics_signal(byte: u8) -> bool {
    byte == MITO_BEACON_SIGNAL || byte == MITO_BEACON_EXTENDED || byte == NUCLEAR_LINEAGE_SIGNAL
}
