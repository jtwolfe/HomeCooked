//! HAL errors.

use core::fmt;

/// Failure reading or writing a logical HAL channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Channel id failed validation.
    InvalidChannel { channel: String, detail: String },
    /// Prefix is not a known [`crate::ChannelKind`].
    UnknownKind { channel: String, prefix: String },
    /// Channel is not registered / not present on this HAL instance.
    UnknownChannel { channel: String },
    /// Operation does not match the channel kind (e.g. write to DI).
    KindMismatch {
        channel: String,
        expected: String,
        detail: String,
    },
    /// Value type does not match the channel (bool vs number).
    TypeMismatch { channel: String, detail: String },
    /// Hardware or simulated fault.
    Fault { channel: String, detail: String },
    /// Software interlock denied the write (when wired).
    InterlockDenied { channel: String, reason: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChannel { channel, detail } => {
                write!(f, "invalid channel {channel:?}: {detail}")
            }
            Self::UnknownKind { channel, prefix } => {
                write!(f, "unknown kind prefix {prefix:?} on channel {channel:?}")
            }
            Self::UnknownChannel { channel } => write!(f, "unknown channel {channel:?}"),
            Self::KindMismatch {
                channel,
                expected,
                detail,
            } => write!(
                f,
                "kind mismatch on {channel:?} (expected {expected}): {detail}"
            ),
            Self::TypeMismatch { channel, detail } => {
                write!(f, "type mismatch on {channel:?}: {detail}")
            }
            Self::Fault { channel, detail } => write!(f, "fault on {channel:?}: {detail}"),
            Self::InterlockDenied { channel, reason } => {
                write!(f, "interlock denied {channel:?}: {reason}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
