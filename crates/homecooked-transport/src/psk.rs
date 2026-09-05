//! Optional lab PSK pairing (binding-level, not TLS/OAuth).
//!
//! # Framing choice
//!
//! When the server has a PSK configured, the **first** length-prefixed JSON
//! frame on each TCP connection is a transport auth preamble — **not** a
//! protocol [`Envelope`](homecooked_protocol::Envelope). Subsequent frames
//! are normal envelopes.
//!
//! ```text
//! Client → Server: {"hc_tcp":"auth","v":1,"psk":"<shared-secret>"}
//! Server → Client: {"hc_tcp":"auth_ok","v":1}
//!   or on failure: {"hc_tcp":"auth_err","v":1,"code":"unauthorized","message":"..."}
//!                  then the server closes the connection.
//! ```
//!
//! Same `[u32 BE length][UTF-8 JSON]` framing as envelopes (`frame` module).
//! Chosen over adding `auth` / `auth_ok` protocol kinds so the catalog stays
//! transport-agnostic (overview §9.1 `unauthorized` is binding-level) and
//! open-lab servers (no PSK) keep today's behaviour with zero preamble.
//!
//! **Lab only:** the shared secret is sent in cleartext over cleartext TCP.
//! TLS / OAuth remain out of scope.

use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::error::TransportError;
use crate::frame::MAX_FRAME_BYTES;

/// Env var for optional lab PSK (`ServerConfig::from_env`, client helpers).
pub const PSK_ENV: &str = "HOMECOOKED_TCP_PSK";

const AUTH_V: u32 = 1;

/// Wire JSON for the auth request preamble.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthRequest {
    pub hc_tcp: AuthTag,
    pub v: u32,
    pub psk: String,
}

/// Wire JSON for a successful auth response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthOk {
    pub hc_tcp: AuthOkTag,
    pub v: u32,
}

/// Wire JSON for a failed auth response (then peer closes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthErr {
    pub hc_tcp: AuthErrTag,
    pub v: u32,
    pub code: String,
    pub message: String,
}

/// Discriminator: `"auth"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthTag {
    Auth,
}

/// Discriminator: `"auth_ok"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthOkTag {
    AuthOk,
}

/// Discriminator: `"auth_err"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthErrTag {
    AuthErr,
}

impl AuthRequest {
    pub fn new(psk: impl Into<String>) -> Self {
        Self {
            hc_tcp: AuthTag::Auth,
            v: AUTH_V,
            psk: psk.into(),
        }
    }
}

impl AuthOk {
    pub fn new() -> Self {
        Self {
            hc_tcp: AuthOkTag::AuthOk,
            v: AUTH_V,
        }
    }
}

impl Default for AuthOk {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthErr {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            hc_tcp: AuthErrTag::AuthErr,
            v: AUTH_V,
            code: "unauthorized".into(),
            message: message.into(),
        }
    }
}

/// Optional shared secret for lab TCP pairing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerConfig {
    /// When `Some`, refuse clients that do not complete PSK auth.
    pub psk: Option<String>,
}

impl ServerConfig {
    /// Open lab: no PSK required (today's behaviour).
    pub fn open() -> Self {
        Self { psk: None }
    }

    /// Require this shared secret on every new connection.
    pub fn with_psk(psk: impl Into<String>) -> Self {
        Self {
            psk: Some(psk.into()),
        }
    }

    /// Read `HOMECOOKED_TCP_PSK` if set and non-empty; otherwise open.
    pub fn from_env() -> Self {
        match std::env::var(PSK_ENV) {
            Ok(s) if !s.is_empty() => Self::with_psk(s),
            _ => Self::open(),
        }
    }

    pub fn psk_required(&self) -> bool {
        self.psk.is_some()
    }
}

/// Constant-time-ish equality for lab secrets (length mismatch → false).
pub fn psk_matches(expected: &str, offered: &str) -> bool {
    let a = expected.as_bytes();
    let b = offered.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn write_json_frame<W: Write>(writer: &mut W, json: &str) -> Result<(), TransportError> {
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

fn read_json_frame<R: Read>(reader: &mut R) -> Result<String, TransportError> {
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
    String::from_utf8(buf).map_err(|e| {
        TransportError::Protocol(homecooked_protocol::ProtocolError::Json(format!(
            "invalid utf-8 in auth frame: {e}"
        )))
    })
}

/// Client → server: send auth request preamble.
pub fn write_auth_request<W: Write>(writer: &mut W, psk: &str) -> Result<(), TransportError> {
    let req = AuthRequest::new(psk);
    let json = serde_json::to_string(&req).map_err(|e| {
        TransportError::Protocol(homecooked_protocol::ProtocolError::Json(e.to_string()))
    })?;
    write_json_frame(writer, &json)
}

/// Server ← client: read auth request preamble.
pub fn read_auth_request<R: Read>(reader: &mut R) -> Result<AuthRequest, TransportError> {
    let json = read_json_frame(reader)?;
    serde_json::from_str(&json).map_err(|e| {
        TransportError::Auth(format!("expected hc_tcp auth preamble as first frame: {e}"))
    })
}

/// Server → client: auth success.
pub fn write_auth_ok<W: Write>(writer: &mut W) -> Result<(), TransportError> {
    let ok = AuthOk::new();
    let json = serde_json::to_string(&ok).map_err(|e| {
        TransportError::Protocol(homecooked_protocol::ProtocolError::Json(e.to_string()))
    })?;
    write_json_frame(writer, &json)
}

/// Server → client: auth failure.
pub fn write_auth_err<W: Write>(
    writer: &mut W,
    message: impl Into<String>,
) -> Result<(), TransportError> {
    let err = AuthErr::unauthorized(message);
    let json = serde_json::to_string(&err).map_err(|e| {
        TransportError::Protocol(homecooked_protocol::ProtocolError::Json(e.to_string()))
    })?;
    write_json_frame(writer, &json)
}

/// Client ← server: read auth response (`auth_ok` or `auth_err`).
pub fn read_auth_response<R: Read>(reader: &mut R) -> Result<(), TransportError> {
    let json = read_json_frame(reader)?;
    if let Ok(ok) = serde_json::from_str::<AuthOk>(&json) {
        if ok.v == AUTH_V {
            return Ok(());
        }
        return Err(TransportError::Auth(format!(
            "unsupported auth_ok version {}",
            ok.v
        )));
    }
    if let Ok(err) = serde_json::from_str::<AuthErr>(&json) {
        return Err(TransportError::Auth(format!(
            "{}: {}",
            err.code, err.message
        )));
    }
    Err(TransportError::Auth(format!(
        "expected auth_ok or auth_err preamble, got: {json}"
    )))
}

/// Perform server-side PSK check; on failure write `auth_err` and return Err.
pub fn server_handshake<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    expected_psk: &str,
) -> Result<(), TransportError> {
    let req = match read_auth_request(reader) {
        Ok(r) => r,
        Err(e) => {
            let _ = write_auth_err(writer, "missing or malformed PSK auth preamble");
            return Err(e);
        }
    };
    if req.v != AUTH_V {
        let _ = write_auth_err(writer, format!("unsupported auth version {}", req.v));
        return Err(TransportError::Auth(format!(
            "unsupported auth version {}",
            req.v
        )));
    }
    if !psk_matches(expected_psk, &req.psk) {
        let _ = write_auth_err(writer, "PSK mismatch");
        return Err(TransportError::Auth("PSK mismatch".into()));
    }
    write_auth_ok(writer)
}

/// Client-side: send PSK and expect `auth_ok`.
pub fn client_handshake<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    psk: &str,
) -> Result<(), TransportError> {
    write_auth_request(writer, psk)?;
    read_auth_response(reader)
}

/// Optional PSK from `HOMECOOKED_TCP_PSK` (None if unset/empty).
pub fn psk_from_env() -> Option<String> {
    match std::env::var(PSK_ENV) {
        Ok(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn psk_matches_equal() {
        assert!(psk_matches("lab-secret", "lab-secret"));
        assert!(!psk_matches("lab-secret", "lab-secre"));
        assert!(!psk_matches("a", "b"));
    }

    #[test]
    fn auth_roundtrip_frames() {
        let mut buf = Vec::new();
        write_auth_request(&mut buf, "s3cret").unwrap();
        let req = read_auth_request(&mut buf.as_slice()).unwrap();
        assert_eq!(req.psk, "s3cret");
        assert_eq!(req.hc_tcp, AuthTag::Auth);

        let mut buf2 = Vec::new();
        write_auth_ok(&mut buf2).unwrap();
        read_auth_response(&mut buf2.as_slice()).unwrap();
    }
}
