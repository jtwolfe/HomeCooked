//! Pluggable request dispatch for the lab TCP server.
//!
//! The default path uses [`homecooked_sim::Simulator`]. Controllers (and other
//! lab backends) implement [`RequestHandler`] so the same framing / accept loop
//! can drive interlock-gated HAL writes without going through the catalog sim.

use homecooked_protocol::Envelope;
use homecooked_sim::Simulator;

/// Handle one HomeCooked request envelope (lab TCP dispatch target).
pub trait RequestHandler: Send {
    fn handle(&mut self, request: Envelope) -> Envelope;
}

impl RequestHandler for Simulator {
    fn handle(&mut self, request: Envelope) -> Envelope {
        Simulator::handle(self, request)
    }
}
