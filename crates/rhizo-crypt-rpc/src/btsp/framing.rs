// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Length-prefixed BTSP frame codec.
//!
//! ```text
//! [ Length: 4 bytes BE u32 ][ Payload: Length bytes ]
//! ```
//!
//! Max frame size: 16 MiB (`MAX_FRAME_SIZE`).
//!
//! Uses `bytes::Bytes` for zero-copy payload handling. Callers receive
//! a reference-counted, cheaply cloneable buffer that avoids copying
//! on the read path.

use std::io;

use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::types::MAX_FRAME_SIZE;

/// Read a single length-prefixed frame from the stream.
///
/// Returns the payload as a frozen `Bytes` buffer, enabling zero-copy
/// downstream processing (CBOR decode, JSON-RPC dispatch, etc.).
///
/// # Errors
///
/// - `UnexpectedEof` if the stream closes before a complete frame.
/// - `InvalidData` if the frame exceeds `MAX_FRAME_SIZE`.
pub async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Bytes> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf);

    if len > MAX_FRAME_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("BTSP frame too large: {len} bytes (max {MAX_FRAME_SIZE})"),
        ));
    }

    let mut payload = BytesMut::zeroed(len as usize);
    reader.read_exact(&mut payload).await?;
    Ok(payload.freeze())
}

/// Write a single length-prefixed frame to the stream.
///
/// # Errors
///
/// - `InvalidData` if the payload exceeds `MAX_FRAME_SIZE`.
/// - I/O errors from the underlying stream.
pub async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    let len: u32 = payload.len().try_into().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("BTSP frame payload too large: {} bytes", payload.len()),
        )
    })?;

    if len > MAX_FRAME_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("BTSP frame too large: {len} bytes (max {MAX_FRAME_SIZE})"),
        ));
    }

    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn round_trip_single_frame() {
        let payload = b"hello BTSP";
        let mut buf = Vec::new();

        write_frame(&mut buf, payload).await.expect("write");
        assert_eq!(buf.len(), 4 + payload.len());

        let mut cursor = io::Cursor::new(buf);
        let read_back = read_frame(&mut cursor).await.expect("read");
        assert_eq!(read_back.as_ref(), payload);
    }

    #[tokio::test]
    async fn round_trip_empty_frame() {
        let mut buf = Vec::new();
        write_frame(&mut buf, b"").await.expect("write");

        let mut cursor = io::Cursor::new(buf);
        let read_back = read_frame(&mut cursor).await.expect("read");
        assert!(read_back.is_empty());
    }

    #[tokio::test]
    async fn rejects_oversized_frame_on_read() {
        let bad_len = (MAX_FRAME_SIZE + 1).to_be_bytes();
        let mut cursor = io::Cursor::new(bad_len.to_vec());
        let err = read_frame(&mut cursor).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("too large"));
    }

    #[tokio::test]
    async fn read_frame_eof_returns_error() {
        let mut cursor = io::Cursor::new(vec![0u8, 0, 0]);
        let err = read_frame(&mut cursor).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }
}
