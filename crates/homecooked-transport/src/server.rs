//! TCP host server: accept connections, dispatch via [`homecooked_sim::Simulator`].

use std::io::{BufReader, ErrorKind};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use homecooked_protocol::Envelope;
use homecooked_sim::Simulator;

use crate::error::TransportError;
use crate::frame::{read_envelope, write_envelope};

/// Shared simulator hub behind the TCP server.
pub type SharedSim = Arc<Mutex<Simulator>>;

/// Result of [`spawn_server`]: bind address, shared sim, accept-loop join handle.
pub type SpawnedServer = (
    SocketAddr,
    SharedSim,
    JoinHandle<Result<(), TransportError>>,
);

/// Wrap a [`Simulator`] for concurrent connection handlers.
pub fn shared_sim(sim: Simulator) -> SharedSim {
    Arc::new(Mutex::new(sim))
}

/// Bind a TCP listener (typically `127.0.0.1:0` in tests).
pub fn bind(addr: impl ToSocketAddrs) -> Result<TcpListener, TransportError> {
    let listener = TcpListener::bind(addr)?;
    Ok(listener)
}

/// Handle one accepted connection until the peer closes or a hard error occurs.
///
/// Request/response: read one framed envelope, dispatch through the sim
/// registry (`Simulator::handle` → `DeviceHub`), write the response frame.
/// Continues until EOF on the next read.
pub fn serve_connection(stream: TcpStream, sim: &SharedSim) -> Result<(), TransportError> {
    stream.set_nodelay(true)?;
    // Idle clients should not hang forever in lab demos.
    let _ = stream.set_read_timeout(Some(Duration::from_secs(60)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

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
        let response = dispatch(&request, sim);
        write_envelope(&mut writer, &response)?;
    }
}

fn dispatch(request: &Envelope, sim: &SharedSim) -> Envelope {
    let mut guard = match sim.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.handle(request.clone())
}

/// Accept loop: one OS thread per connection.
///
/// Returns when the listener is dropped / accept fails with a non-temporary error.
pub fn accept_loop(listener: TcpListener, sim: SharedSim) -> Result<(), TransportError> {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let sim = Arc::clone(&sim);
                thread::spawn(move || {
                    if let Err(e) = serve_connection(stream, &sim) {
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

/// Spawn [`accept_loop`] on a background thread. Returns local bind address + join handle.
pub fn spawn_server(
    addr: impl ToSocketAddrs,
    sim: Simulator,
) -> Result<SpawnedServer, TransportError> {
    let listener = bind(addr)?;
    let local = listener.local_addr()?;
    let shared = shared_sim(sim);
    let sim_for_loop = Arc::clone(&shared);
    let handle = thread::spawn(move || accept_loop(listener, sim_for_loop));
    Ok((local, shared, handle))
}

/// Serve a single connection on the calling thread (useful for tests).
pub fn serve_one(listener: &TcpListener, sim: &SharedSim) -> Result<(), TransportError> {
    let (stream, _) = listener.accept()?;
    serve_connection(stream, sim)
}
