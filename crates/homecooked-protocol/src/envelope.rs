//! Message envelope: version, ids, and kind-tagged body.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use homecooked_schema::SemVer;
use serde::{Deserialize, Serialize};

use crate::error::{ErrorBody, ProtocolError};
use crate::messages::Payload;
use crate::PROTOCOL_VERSION;

static MESSAGE_SEQ: AtomicU64 = AtomicU64::new(1);

/// Monotonic message id (`m-<n>`). Not a UUID; sufficient for in-process use.
pub fn next_message_id() -> String {
    format!("m-{}", MESSAGE_SEQ.fetch_add(1, Ordering::Relaxed))
}

/// Sender timestamp in milliseconds since Unix epoch. Returns 0 if the clock
/// is before the epoch.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Every message on the wire (overview §6.2).
///
/// Field names follow the PR3 crate sketch (`protocol_version`, `message_id`,
/// optional `correlation_id`) with serde aliases for the overview names
/// (`proto`, `id`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    #[serde(alias = "proto")]
    pub protocol_version: SemVer,
    #[serde(alias = "id")]
    pub message_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub ts_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
    #[serde(flatten)]
    pub payload: Payload,
}

impl Envelope {
    pub fn new(payload: Payload) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            message_id: next_message_id(),
            correlation_id: None,
            ts_ms: now_ms(),
            device_id: None,
            timeout_ms: None,
            payload,
        }
    }

    pub fn request(device_id: Option<String>, payload: Payload) -> Self {
        let mut env = Self::new(payload);
        env.device_id = device_id;
        env
    }

    /// Build a success (or error) response that echoes the request correlation.
    pub fn respond_to(request: &Self, payload: Payload) -> Self {
        let mut env = Self::new(payload);
        env.correlation_id = Some(
            request
                .correlation_id
                .clone()
                .unwrap_or_else(|| request.message_id.clone()),
        );
        env.device_id = request.device_id.clone();
        env
    }

    pub fn error_to(request: &Self, body: ErrorBody) -> Self {
        Self::respond_to(request, Payload::Error(body))
    }

    pub fn with_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn with_ts_ms(mut self, ts_ms: u64) -> Self {
        self.ts_ms = ts_ms;
        self
    }

    pub fn kind(&self) -> crate::messages::MessageKind {
        self.payload.kind()
    }

    /// Clients reject a peer only on protocol **major** mismatch (§8.2).
    pub fn check_protocol_version(&self) -> Result<(), ProtocolError> {
        check_protocol_version(self.protocol_version)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Decode JSON and reject protocol major mismatch.
    pub fn decode(s: &str) -> Result<Self, ProtocolError> {
        let env = Self::from_json(s)?;
        env.check_protocol_version()?;
        Ok(env)
    }
}

pub fn check_protocol_version(got: SemVer) -> Result<(), ProtocolError> {
    if got.major != PROTOCOL_VERSION.major {
        Err(ProtocolError::version_mismatch(got))
    } else {
        Ok(())
    }
}
