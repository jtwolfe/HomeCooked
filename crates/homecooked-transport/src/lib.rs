//! TCP transport for HomeCooked protocol envelopes (lab / host path).
//!
//! # Framing
//!
//! Length-prefixed JSON — see [`frame`] for the on-wire layout and rationale
//! (chosen over NDJSON for binary-safe boundaries; overview §6.1).
//!
//! # Lab PSK pairing
//!
//! Optional shared-secret handshake so anonymous TCP clients can be refused
//! in lab setups. See [`psk`]: first frame is a dedicated auth preamble when
//! the server has a PSK; otherwise behaviour matches the open lab (no auth).
//!
//! # Scope
//!
//! - Host server: accept TCP, decode request, dispatch via
//!   [`homecooked_sim::Simulator`] **or** any [`RequestHandler`] (e.g. a
//!   controller-sim endpoint), encode response.
//! - Client helper: connect, send Discover / Describe / Read / Write, read response.
//! - **Optional lab PSK** (cleartext shared secret). **No TLS, no OAuth**.
//!
//! Roadmap: Stream 4 milestone 3 (`docs/ROADMAP.md`).

mod client;
mod error;
pub mod frame;
mod handler;
pub mod psk;
mod server;

pub use client::{TcpClient, DEFAULT_TIMEOUT};
pub use error::TransportError;
pub use frame::{read_envelope, write_envelope, MAX_FRAME_BYTES};
pub use handler::RequestHandler;
pub use psk::{psk_from_env, ServerConfig, PSK_ENV};
pub use server::{
    accept_handler_loop, accept_loop, bind, serve_connection, serve_handler_connection, serve_one,
    shared_handler, shared_sim, spawn_handler_server, spawn_handler_server_with_config,
    spawn_server, spawn_server_with_config, SharedHandler, SharedSim, SpawnedHandlerServer,
    SpawnedServer,
};
