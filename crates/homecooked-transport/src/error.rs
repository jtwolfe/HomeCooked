//! Transport errors (I/O, framing, protocol decode).

use std::fmt;
use std::io;

use homecooked_protocol::ProtocolError;
use homecooked_schema::ErrorCode;

/// Errors from framing, TCP I/O, or protocol decode on the wire.
#[derive(Debug)]
pub enum TransportError {
    Io(io::Error),
    Protocol(ProtocolError),
    /// Framed length exceeds [`crate::frame::MAX_FRAME_BYTES`].
    FrameTooLarge {
        len: u32,
    },
    /// Zero-length frame (not a valid envelope).
    EmptyFrame,
    /// Peer closed the connection mid-frame or before a response.
    UnexpectedEof,
    /// Response kind was `error` (or unexpected) when a success payload was required.
    Remote(homecooked_protocol::ErrorBody),
    /// Success payload kind did not match the request.
    UnexpectedKind {
        expected: &'static str,
        got: String,
    },
}

impl TransportError {
    pub fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::Protocol(p) => Some(p.to_error_body().code),
            Self::Remote(body) => Some(body.code),
            Self::FrameTooLarge { .. } | Self::EmptyFrame => Some(ErrorCode::InvalidRequest),
            Self::UnexpectedEof | Self::Io(_) => Some(ErrorCode::Timeout),
            Self::UnexpectedKind { .. } => Some(ErrorCode::Internal),
        }
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "tcp i/o: {e}"),
            Self::Protocol(e) => write!(f, "protocol: {e}"),
            Self::FrameTooLarge { len } => write!(f, "frame too large: {len} bytes"),
            Self::EmptyFrame => write!(f, "empty frame"),
            Self::UnexpectedEof => write!(f, "unexpected eof"),
            Self::Remote(body) => write!(f, "remote {}: {}", body.code.as_str(), body.message),
            Self::UnexpectedKind { expected, got } => {
                write!(
                    f,
                    "unexpected response kind: expected {expected}, got {got}"
                )
            }
        }
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Protocol(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for TransportError {
    fn from(value: io::Error) -> Self {
        if value.kind() == io::ErrorKind::UnexpectedEof {
            Self::UnexpectedEof
        } else {
            Self::Io(value)
        }
    }
}

impl From<ProtocolError> for TransportError {
    fn from(value: ProtocolError) -> Self {
        Self::Protocol(value)
    }
}
