//! Kind-specific request, response, and event bodies.

use homecooked_schema::{
    ApplianceClassId, CapabilityModel, CatalogVersion, QualifiedPointId, SemVer, TraitId, Value,
    CATALOG_VERSION,
};
use serde::{Deserialize, Serialize};

use crate::error::ErrorBody;
use crate::PROTOCOL_VERSION;

/// Wire message kind tokens (overview §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    Discover,
    DiscoverOk,
    Describe,
    DescribeOk,
    Read,
    ReadOk,
    Write,
    WriteOk,
    Subscribe,
    SubscribeOk,
    Unsubscribe,
    UnsubscribeOk,
    Event,
    Error,
    Ping,
    Pong,
    CapsChanged,
}

impl MessageKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discover => "discover",
            Self::DiscoverOk => "discover_ok",
            Self::Describe => "describe",
            Self::DescribeOk => "describe_ok",
            Self::Read => "read",
            Self::ReadOk => "read_ok",
            Self::Write => "write",
            Self::WriteOk => "write_ok",
            Self::Subscribe => "subscribe",
            Self::SubscribeOk => "subscribe_ok",
            Self::Unsubscribe => "unsubscribe",
            Self::UnsubscribeOk => "unsubscribe_ok",
            Self::Event => "event",
            Self::Error => "error",
            Self::Ping => "ping",
            Self::Pong => "pong",
            Self::CapsChanged => "caps_changed",
        }
    }

    pub const fn is_request(self) -> bool {
        matches!(
            self,
            Self::Discover
                | Self::Describe
                | Self::Read
                | Self::Write
                | Self::Subscribe
                | Self::Unsubscribe
                | Self::Ping
        )
    }
}

impl std::fmt::Display for MessageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Kind-tagged body. Flattened onto [`crate::Envelope`] as `kind` + `body`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "body", rename_all = "snake_case")]
pub enum Payload {
    Discover(DiscoverRequest),
    DiscoverOk(DiscoverResponse),
    Describe(DescribeRequest),
    DescribeOk(Box<DescribeResponse>),
    Read(ReadRequest),
    ReadOk(ReadResponse),
    Write(WriteRequest),
    WriteOk(WriteResponse),
    Subscribe(SubscribeRequest),
    SubscribeOk(SubscribeResponse),
    Unsubscribe(UnsubscribeRequest),
    UnsubscribeOk(UnsubscribeResponse),
    Event(EventBody),
    Error(ErrorBody),
    Ping(PingBody),
    Pong(PongBody),
    CapsChanged(CapsChangedBody),
}

impl Payload {
    pub fn kind(&self) -> MessageKind {
        match self {
            Self::Discover(_) => MessageKind::Discover,
            Self::DiscoverOk(_) => MessageKind::DiscoverOk,
            Self::Describe(_) => MessageKind::Describe,
            Self::DescribeOk(_) => MessageKind::DescribeOk,
            Self::Read(_) => MessageKind::Read,
            Self::ReadOk(_) => MessageKind::ReadOk,
            Self::Write(_) => MessageKind::Write,
            Self::WriteOk(_) => MessageKind::WriteOk,
            Self::Subscribe(_) => MessageKind::Subscribe,
            Self::SubscribeOk(_) => MessageKind::SubscribeOk,
            Self::Unsubscribe(_) => MessageKind::Unsubscribe,
            Self::UnsubscribeOk(_) => MessageKind::UnsubscribeOk,
            Self::Event(_) => MessageKind::Event,
            Self::Error(_) => MessageKind::Error,
            Self::Ping(_) => MessageKind::Ping,
            Self::Pong(_) => MessageKind::Pong,
            Self::CapsChanged(_) => MessageKind::CapsChanged,
        }
    }
}

/// Hello-class advertisement (overview §5.1 / §7.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloRecord {
    pub device_id: String,
    pub protocol_version: SemVer,
    pub catalog_version: CatalogVersion,
    pub class_id: ApplianceClassId,
    pub trait_ids: Vec<TraitId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

impl HelloRecord {
    pub fn new(device_id: impl Into<String>, class_id: ApplianceClassId) -> Self {
        Self {
            device_id: device_id.into(),
            protocol_version: PROTOCOL_VERSION,
            catalog_version: CATALOG_VERSION,
            class_id,
            trait_ids: Vec::new(),
            display_name: None,
            endpoint: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DiscoverRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_id: Option<ApplianceClassId>,
    /// Device must advertise all listed traits. Empty = no trait filter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trait_ids: Vec<TraitId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DiscoverResponse {
    pub devices: Vec<HelloRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DescribeRequest {
    /// Empty = full capability object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<QualifiedPointId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DescribeResponse {
    pub capability: CapabilityModel,
}

/// Recommended maximum point ids per read (overview §7.3).
pub const MAX_READ_POINTS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReadRequest {
    pub points: Vec<QualifiedPointId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub allow_partial: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointValue {
    pub id: QualifiedPointId,
    /// `null` only for optional points that are currently unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default)]
    pub ts_ms: u64,
}

impl PointValue {
    pub fn new(id: QualifiedPointId, value: Value, ts_ms: u64) -> Self {
        Self {
            id,
            value: Some(value),
            ts_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ReadResponse {
    pub values: Vec<PointValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorBody>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteOp {
    pub id: QualifiedPointId,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct WriteRequest {
    pub writes: Vec<WriteOp>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dry_run: bool,
    /// Default false. Devices may reject `true` with `unsupported_operation`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub atomic: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct WriteResponse {
    pub accepted: Vec<WriteOp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventReason {
    Value,
    Cycle,
    Fault,
    CapsChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SubscribeRequest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<QualifiedPointId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<TraitId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub all: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_period_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<EventReason>,
}

impl SubscribeRequest {
    pub fn is_empty(&self) -> bool {
        self.points.is_empty() && self.traits.is_empty() && !self.all
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SubscribeResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<QualifiedPointId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<TraitId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub all: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UnsubscribeRequest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<QualifiedPointId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<TraitId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub all: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UnsubscribeResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<QualifiedPointId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FaultSnapshot {
    #[serde(default)]
    pub fault_present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault_severity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alert_list: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventBody {
    pub reason: EventReason,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<PointValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault: Option<FaultSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PingBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PongBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CapsChangedBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
