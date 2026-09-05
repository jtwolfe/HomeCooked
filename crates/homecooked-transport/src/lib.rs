//! TCP transport for HomeCooked protocol envelopes (lab / host path).
//!
//! # Framing
//!
//! Length-prefixed JSON — see [`frame`] for the on-wire layout and rationale
//! (chosen over NDJSON for binary-safe boundaries; overview §6.1).
//!
//! # Scope
//!
//! - Host server: accept TCP, decode request, dispatch via
//!   [`homecooked_sim::Simulator`] (registry + capability checks), encode response.
//! - Client helper: connect, send Discover / Describe / Read / Write, read response.
//! - **No TLS, no OAuth / device auth** — lab-only cleartext TCP.
//!
//! Roadmap: Stream 4 milestone 3 (`docs/ROADMAP.md`).

mod client;
mod error;
pub mod frame;
mod server;

pub use client::{TcpClient, DEFAULT_TIMEOUT};
pub use error::TransportError;
pub use frame::{read_envelope, write_envelope, MAX_FRAME_BYTES};
pub use server::{
    accept_loop, bind, serve_connection, serve_one, shared_sim, spawn_server, SharedSim,
    SpawnedServer,
};
