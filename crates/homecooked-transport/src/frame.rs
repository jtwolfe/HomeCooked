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
    use std::io::Cursor;

    use homecooked_protocol::{Envelope, Payload, PingBody, ProtocolError};

    use super::*;

    fn frame_bytes(payload: &[u8]) -> Vec<u8> {
        let mut out = (payload.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(payload);
        out
    }

    #[derive(Debug, Clone, Copy)]
    enum Expect {
        FrameTooLarge,
        EmptyFrame,
        UnexpectedEof,
        ProtocolJson,
        ProtocolUtf8,
    }

    fn matches_expect(err: &TransportError, expect: Expect) -> bool {
        match expect {
            Expect::FrameTooLarge => {
                matches!(err, TransportError::FrameTooLarge { len } if *len == MAX_FRAME_BYTES + 1)
            }
            Expect::EmptyFrame => matches!(err, TransportError::EmptyFrame),
            Expect::UnexpectedEof => matches!(err, TransportError::UnexpectedEof),
            Expect::ProtocolJson => {
                matches!(err, TransportError::Protocol(ProtocolError::Json(_)))
            }
            Expect::ProtocolUtf8 => matches!(
                err,
                TransportError::Protocol(ProtocolError::Json(msg)) if msg.contains("invalid utf-8")
            ),
        }
    }

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

    /// Table-driven malformed length-prefixed frames (tooling / conformance).
    ///
    /// Covers oversize length, empty frame, truncated header/body, invalid
    /// UTF-8, truncated/invalid JSON, and unknown `kind` — without `cargo fuzz`
    /// (deferred; see `docs/ROADMAP.md`).
    #[test]
    fn malformed_frames_table() {
        // Valid minimal ping JSON for positive control.
        let good_json =
            br#"{"protocol_version":"0.1.0","message_id":"m-1","ts_ms":0,"kind":"ping","body":{}}"#;

        let mut oversize = (MAX_FRAME_BYTES + 1).to_be_bytes().to_vec();
        oversize.extend_from_slice(&[0u8; 8]);

        let mut truncated_body = 32u32.to_be_bytes().to_vec();
        truncated_body.extend_from_slice(br#"{"a":1}"#);

        let cases: &[(&str, Vec<u8>, Expect)] = &[
            ("oversize_length", oversize, Expect::FrameTooLarge),
            ("empty_frame", frame_bytes(&[]), Expect::EmptyFrame),
            ("truncated_length_header", vec![0x00, 0x00], Expect::UnexpectedEof),
            ("truncated_body", truncated_body, Expect::UnexpectedEof),
            (
                "invalid_utf8",
                frame_bytes(&[0xff, 0xfe, 0xfd, 0xfc]),
                Expect::ProtocolUtf8,
            ),
            (
                "truncated_json",
                frame_bytes(br#"{"protocol_version":"0.1.0","message_id":"m-1","kind":"ping""#),
                Expect::ProtocolJson,
            ),
            (
                "invalid_json_not_object",
                frame_bytes(br#"[1,2,3]"#),
                Expect::ProtocolJson,
            ),
            (
                "unknown_kind",
                frame_bytes(
                    br#"{"protocol_version":"0.1.0","message_id":"m-1","ts_ms":0,"kind":"not_a_real_kind","body":{}}"#,
                ),
                Expect::ProtocolJson,
            ),
            (
                "missing_kind",
                frame_bytes(
                    br#"{"protocol_version":"0.1.0","message_id":"m-1","ts_ms":0,"body":{}}"#,
                ),
                Expect::ProtocolJson,
            ),
        ];

        for (name, bytes, expect) in cases {
            let err = read_envelope(&mut bytes.as_slice()).unwrap_err();
            assert!(
                matches_expect(&err, *expect),
                "case `{name}`: unexpected error {err:?}, expected {expect:?}"
            );
        }

        // Positive: good_json framed must succeed (guards the table harness).
        let ok = read_envelope(&mut frame_bytes(good_json).as_slice()).unwrap();
        assert_eq!(ok.message_id, "m-1");
        assert!(matches!(ok.payload, Payload::Ping(_)));
    }

    #[test]
    fn rejects_oversize_length() {
        let mut bogus = (MAX_FRAME_BYTES + 1).to_be_bytes().to_vec();
        bogus.extend_from_slice(&[0u8; 8]);
        let err = read_envelope(&mut bogus.as_slice()).unwrap_err();
        assert!(matches!(err, TransportError::FrameTooLarge { .. }));
    }

    #[test]
    fn truncated_stream_via_cursor() {
        let mut partial = 8u32.to_be_bytes().to_vec();
        partial.extend_from_slice(br#"{"k":"#);
        let mut cur = Cursor::new(partial);
        let err = read_envelope(&mut cur).unwrap_err();
        assert!(matches!(err, TransportError::UnexpectedEof));
    }
}
