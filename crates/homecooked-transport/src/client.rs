//! TCP client helper: connect, exchange HomeCooked envelopes.

use std::io::{BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use homecooked_protocol::{
    DescribeRequest, DescribeResponse, DiscoverRequest, DiscoverResponse, Envelope, Payload,
    ReadRequest, ReadResponse, WriteOp, WriteRequest, WriteResponse,
};
use homecooked_schema::{ApplianceClassId, QualifiedPointId, TraitId, Value};

use crate::error::TransportError;
use crate::frame::{read_envelope, write_envelope};

/// Default I/O timeout for lab clients (seconds).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Blocking TCP client for request/response exchanges.
pub struct TcpClient {
    stream: TcpStream,
}

impl TcpClient {
    /// Connect to `addr` (e.g. `"127.0.0.1:9876"`).
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, TransportError> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(DEFAULT_TIMEOUT))?;
        stream.set_write_timeout(Some(DEFAULT_TIMEOUT))?;
        stream.set_nodelay(true)?;
        Ok(Self { stream })
    }

    /// Connect with an explicit I/O timeout.
    pub fn connect_timeout(
        addr: impl ToSocketAddrs,
        timeout: Duration,
    ) -> Result<Self, TransportError> {
        let mut last_err = None;
        for a in addr.to_socket_addrs()? {
            match TcpStream::connect_timeout(&a, timeout) {
                Ok(stream) => {
                    stream.set_read_timeout(Some(timeout))?;
                    stream.set_write_timeout(Some(timeout))?;
                    stream.set_nodelay(true)?;
                    return Ok(Self { stream });
                }
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err
            .unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no addresses"))
            .into())
    }

    /// Send `request` and read one response envelope.
    pub fn exchange(&mut self, request: &Envelope) -> Result<Envelope, TransportError> {
        write_envelope(&mut self.stream, request)?;
        self.stream.flush()?;
        let mut reader = BufReader::new(&self.stream);
        read_envelope(&mut reader)
    }

    /// `discover` (optional class / trait filters).
    pub fn discover(
        &mut self,
        class_id: Option<ApplianceClassId>,
        trait_ids: Vec<TraitId>,
    ) -> Result<DiscoverResponse, TransportError> {
        let req = Envelope::new(Payload::Discover(DiscoverRequest {
            class_id,
            trait_ids,
        }));
        match self.exchange(&req)?.payload {
            Payload::DiscoverOk(body) => Ok(body),
            Payload::Error(body) => Err(TransportError::Remote(body)),
            other => Err(TransportError::UnexpectedKind {
                expected: "discover_ok",
                got: other.kind().as_str().to_string(),
            }),
        }
    }

    /// `describe` for `device_id` (empty `points` = full capability).
    pub fn describe(
        &mut self,
        device_id: &str,
        points: Vec<QualifiedPointId>,
    ) -> Result<DescribeResponse, TransportError> {
        let req = Envelope::request(
            Some(device_id.into()),
            Payload::Describe(DescribeRequest { points }),
        );
        match self.exchange(&req)?.payload {
            Payload::DescribeOk(body) => Ok(*body),
            Payload::Error(body) => Err(TransportError::Remote(body)),
            other => Err(TransportError::UnexpectedKind {
                expected: "describe_ok",
                got: other.kind().as_str().to_string(),
            }),
        }
    }

    /// `read` points from `device_id`.
    pub fn read(
        &mut self,
        device_id: &str,
        points: Vec<QualifiedPointId>,
    ) -> Result<ReadResponse, TransportError> {
        let req = Envelope::request(
            Some(device_id.into()),
            Payload::Read(ReadRequest {
                points,
                allow_partial: false,
            }),
        );
        match self.exchange(&req)?.payload {
            Payload::ReadOk(body) => Ok(body),
            Payload::Error(body) => Err(TransportError::Remote(body)),
            other => Err(TransportError::UnexpectedKind {
                expected: "read_ok",
                got: other.kind().as_str().to_string(),
            }),
        }
    }

    /// `write` one or more ops to `device_id`.
    pub fn write(
        &mut self,
        device_id: &str,
        writes: Vec<WriteOp>,
    ) -> Result<WriteResponse, TransportError> {
        let req = Envelope::request(
            Some(device_id.into()),
            Payload::Write(WriteRequest {
                writes,
                dry_run: false,
                atomic: false,
            }),
        );
        match self.exchange(&req)?.payload {
            Payload::WriteOk(body) => Ok(body),
            Payload::Error(body) => Err(TransportError::Remote(body)),
            other => Err(TransportError::UnexpectedKind {
                expected: "write_ok",
                got: other.kind().as_str().to_string(),
            }),
        }
    }

    /// Convenience: write a single point.
    pub fn write_point(
        &mut self,
        device_id: &str,
        point_id: &str,
        value: Value,
    ) -> Result<WriteResponse, TransportError> {
        let id = point_id
            .parse()
            .map_err(|e: homecooked_schema::ParseIdError| {
                TransportError::Remote(homecooked_protocol::ErrorBody::invalid_request(
                    e.to_string(),
                ))
            })?;
        self.write(device_id, vec![WriteOp { id, value }])
    }

    /// Underlying stream (for advanced use / tests).
    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }
}
