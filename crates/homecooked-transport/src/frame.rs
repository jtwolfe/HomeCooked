//! Length-prefixed JSON framing for [`homecooked_protocol::Envelope`].
//!
//! # Framing choice
//!
//! Each message on the TCP stream is:
//!
//! ```text
//! [u32 big-endian length][UTF-8 JSON envelope bytes]
//! ```
//!
//! - Length is the byte length of the JSON payload only (not including the
//!   4-byte header).
//! - Maximum payload size is [`MAX_FRAME_BYTES`] (64 KiB), matching
//!   `docs/standard/overview.md` §6.1.
//! - JSON is the compact encoding from [`Envelope::to_json`] (no pretty-print
//!   newlines inside the frame).
//!
//! **Why length-prefixed instead of newline-delimited (NDJSON)?**
//! Overview §6.1 asks bindings for binary-safe payloads. A u32 length prefix
//! gives unambiguous boundaries without scanning, stays safe if a future
//! binding uses pretty JSON or CBOR, and avoids ambiguity if a payload ever
//! contained a newline. NDJSON would be fine for lab tooling with compact
//! JSON only; this crate prefers the stricter framing for the lab TCP path.

use std::io::{Read, Write};

use homecooked_protocol::{Envelope, ProtocolError};

use crate::error::TransportError;

/// Maximum JSON payload bytes per frame (overview §6.1 minimum binding size).
pub const MAX_FRAME_BYTES: u32 = 64 * 1024;

/// Write one envelope as a length-prefixed JSON frame.
pub fn write_envelope<W: Write>(writer: &mut W, env: &Envelope) -> Result<(), TransportError> {
    let json = env.to_json().map_err(ProtocolError::from)?;
    let len = u32::try_from(json.len()).unwrap_or(u32::MAX);
    if len == 0 {
        return Err(TransportError::EmptyFrame);
    }
    if len > MAX_FRAME_BYTES {
        return Err(TransportError::FrameTooLarge { len });
    }
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(json.as_bytes())?;
    writer.flush()?;
    Ok(())
}

/// Read one length-prefixed JSON envelope from `reader`.
pub fn read_envelope<R: Read>(reader: &mut R) -> Result<Envelope, TransportError> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf);
    if len == 0 {
        return Err(TransportError::EmptyFrame);
    }
    if len > MAX_FRAME_BYTES {
        return Err(TransportError::FrameTooLarge { len });
    }
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;
    let json = std::str::from_utf8(&buf).map_err(|e| {
        TransportError::Protocol(ProtocolError::Json(format!("invalid utf-8 in frame: {e}")))
    })?;
    // Decode + reject protocol major mismatch.
    Ok(Envelope::decode(json)?)
}

#[cfg(test)]
mod tests {
    use homecooked_protocol::{Envelope, Payload, PingBody};

    use super::*;

    #[test]
    fn roundtrip_ping() {
        let env = Envelope::new(Payload::Ping(PingBody {
            echo: Some("lab".into()),
        }));
        let mut buf = Vec::new();
        write_envelope(&mut buf, &env).unwrap();
        assert_eq!(&buf[..4], &(buf.len() as u32 - 4).to_be_bytes());
        let back = read_envelope(&mut buf.as_slice()).unwrap();
        assert_eq!(back.payload, env.payload);
        assert_eq!(back.message_id, env.message_id);
    }

    #[test]
    fn rejects_oversize_length() {
        let mut bogus = (MAX_FRAME_BYTES + 1).to_be_bytes().to_vec();
        bogus.extend_from_slice(&[0u8; 8]);
        let err = read_envelope(&mut bogus.as_slice()).unwrap_err();
        assert!(matches!(err, TransportError::FrameTooLarge { .. }));
    }
}
