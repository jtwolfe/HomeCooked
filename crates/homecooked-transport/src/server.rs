//! TCP host server: accept connections, dispatch via [`homecooked_sim::Simulator`]
//! or any [`crate::RequestHandler`].

use std::io::{BufReader, ErrorKind};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use homecooked_protocol::Envelope;
use homecooked_sim::Simulator;

use crate::error::TransportError;
use crate::frame::{read_envelope, write_envelope};
use crate::handler::RequestHandler;
use crate::psk::{server_handshake, ServerConfig};

/// Shared simulator hub behind the TCP server.
pub type SharedSim = Arc<Mutex<Simulator>>;

/// Shared pluggable request handler behind the TCP server.
pub type SharedHandler = Arc<Mutex<dyn RequestHandler>>;

/// Result of [`spawn_server`]: bind address, shared sim, accept-loop join handle.
pub type SpawnedServer = (
    SocketAddr,
    SharedSim,
    JoinHandle<Result<(), TransportError>>,
);

/// Result of [`spawn_handler_server`]: bind address, shared handler, join handle.
pub type SpawnedHandlerServer = (
    SocketAddr,
    SharedHandler,
    JoinHandle<Result<(), TransportError>>,
);

/// Wrap a [`Simulator`] for concurrent connection handlers.
pub fn shared_sim(sim: Simulator) -> SharedSim {
    Arc::new(Mutex::new(sim))
}

/// Wrap any [`RequestHandler`] for concurrent connection handlers.
pub fn shared_handler(handler: impl RequestHandler + 'static) -> SharedHandler {
    Arc::new(Mutex::new(handler))
}

/// Bind a TCP listener (typically `127.0.0.1:0` in tests).
pub fn bind(addr: impl ToSocketAddrs) -> Result<TcpListener, TransportError> {
    let listener = TcpListener::bind(addr)?;
    Ok(listener)
}

/// Handle one accepted connection until the peer closes or a hard error occurs.
///
/// If `config.psk` is set, the first frame must be a lab PSK auth preamble
/// (see [`crate::psk`]); unauthenticated clients are refused with `auth_err`.
/// Then: read one framed envelope, dispatch through the sim registry
/// (`Simulator::handle` → `DeviceHub`), write the response frame.
/// Continues until EOF on the next read.
pub fn serve_connection(
    stream: TcpStream,
    sim: &SharedSim,
    config: &ServerConfig,
) -> Result<(), TransportError> {
    serve_handler_connection(stream, &sim_as_handler(sim), config)
}

/// Like [`serve_connection`], but dispatches through any [`RequestHandler`].
pub fn serve_handler_connection(
    stream: TcpStream,
    handler: &SharedHandler,
    config: &ServerConfig,
) -> Result<(), TransportError> {
    stream.set_nodelay(true)?;
    // Idle clients should not hang forever in lab demos.
    let _ = stream.set_read_timeout(Some(Duration::from_secs(60)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    if let Some(psk) = config.psk.as_deref() {
        server_handshake(&mut reader, &mut writer, psk)?;
    }

    loop {
        let request = match read_envelope(&mut reader) {
            Ok(env) => env,
            Err(TransportError::UnexpectedEof) => return Ok(()),
            Err(TransportError::Io(ref e))
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut =>
            {
                // Treat idle timeout as clean disconnect for lab servers.
                return Ok(());
            }
            Err(e) => {
                // Best-effort: if decode failed but we still have a stream, try
                // to surface nothing further — framing errors end the session.
                return Err(e);
            }
        };
        let response = dispatch_handler(&request, handler);
        write_envelope(&mut writer, &response)?;
    }
}

fn sim_as_handler(sim: &SharedSim) -> SharedHandler {
    // Clone the Arc<Mutex<Simulator>> into a trait object by wrapping a
    // thin adapter that locks the same mutex. We cannot coerce
    // Arc<Mutex<Simulator>> to Arc<Mutex<dyn RequestHandler>> directly.
    Arc::new(Mutex::new(SimHandlerProxy {
        inner: Arc::clone(sim),
    }))
}

/// Forwards [`RequestHandler`] calls into a shared [`Simulator`] mutex.
struct SimHandlerProxy {
    inner: SharedSim,
}

impl RequestHandler for SimHandlerProxy {
    fn handle(&mut self, request: Envelope) -> Envelope {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.handle(request)
    }
}

fn dispatch_handler(request: &Envelope, handler: &SharedHandler) -> Envelope {
    let mut guard = match handler.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.handle(request.clone())
}

/// Accept loop: one OS thread per connection (sim-backed).
///
/// Returns when the listener is dropped / accept fails with a non-temporary error.
pub fn accept_loop(
    listener: TcpListener,
    sim: SharedSim,
    config: ServerConfig,
) -> Result<(), TransportError> {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let sim = Arc::clone(&sim);
                let config = config.clone();
                thread::spawn(move || {
                    if let Err(e) = serve_connection(stream, &sim, &config) {
                        eprintln!("homecooked-transport: connection error: {e}");
                    }
                });
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// Accept loop for a pluggable [`RequestHandler`].
pub fn accept_handler_loop(
    listener: TcpListener,
    handler: SharedHandler,
    config: ServerConfig,
) -> Result<(), TransportError> {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let handler = Arc::clone(&handler);
                let config = config.clone();
                thread::spawn(move || {
                    if let Err(e) = serve_handler_connection(stream, &handler, &config) {
                        eprintln!("homecooked-transport: connection error: {e}");
                    }
                });
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

/// Spawn [`accept_loop`] on a background thread (open lab: no PSK).
///
/// Equivalent to [`spawn_server_with_config`] with [`ServerConfig::open`].
pub fn spawn_server(
    addr: impl ToSocketAddrs,
    sim: Simulator,
) -> Result<SpawnedServer, TransportError> {
    spawn_server_with_config(addr, sim, ServerConfig::open())
}

/// Spawn accept loop with optional lab PSK ([`ServerConfig`]).
pub fn spawn_server_with_config(
    addr: impl ToSocketAddrs,
    sim: Simulator,
    config: ServerConfig,
) -> Result<SpawnedServer, TransportError> {
    let listener = bind(addr)?;
    let local = listener.local_addr()?;
    let shared = shared_sim(sim);
    let sim_for_loop = Arc::clone(&shared);
    let handle = thread::spawn(move || accept_loop(listener, sim_for_loop, config));
    Ok((local, shared, handle))
}

/// Spawn accept loop for any [`RequestHandler`] (open lab: no PSK).
pub fn spawn_handler_server(
    addr: impl ToSocketAddrs,
    handler: impl RequestHandler + 'static,
) -> Result<SpawnedHandlerServer, TransportError> {
    spawn_handler_server_with_config(addr, handler, ServerConfig::open())
}

/// Spawn accept loop for any [`RequestHandler`] with optional lab PSK.
pub fn spawn_handler_server_with_config(
    addr: impl ToSocketAddrs,
    handler: impl RequestHandler + 'static,
    config: ServerConfig,
) -> Result<SpawnedHandlerServer, TransportError> {
    let listener = bind(addr)?;
    let local = listener.local_addr()?;
    let shared = shared_handler(handler);
    let handler_for_loop = Arc::clone(&shared);
    let handle = thread::spawn(move || accept_handler_loop(listener, handler_for_loop, config));
    Ok((local, shared, handle))
}

/// Serve a single connection on the calling thread (useful for tests).
pub fn serve_one(
    listener: &TcpListener,
    sim: &SharedSim,
    config: &ServerConfig,
) -> Result<(), TransportError> {
    let (stream, _) = listener.accept()?;
    serve_connection(stream, sim, config)
}
