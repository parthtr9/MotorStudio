//! The gateway: sole writer to any CAN bus (per root `CLAUDE.md`), owning
//! arming, enforcement, the watchdog, and the trip log. `daemon` holds
//! the core state machine (arm/disarm/command-gating/watchdog); `server`
//! is the Unix-socket transport on top of it (one request per line, see
//! `protocol`). A WebSocket transport is not built yet -- it would
//! reuse `server::GatewayServer::dispatch` unchanged, only the framing
//! differs.
//!
//! `enforce/` (the directory alongside this file) is deliberately left
//! with no `mod.rs` and is NOT declared as a module here. See
//! `ENFORCE_PROPOSAL.md` in this directory for the proposed clamp/I^2t/
//! watchdog design -- it is a proposal for a human to review and author,
//! not code this scaffolding includes. `.claude/hooks/guard.py` blocks
//! Write/Edit under `driver/src/gateway/enforce/`, so even a stub
//! `mod.rs` there has to come from a human, not an agent -- which is also
//! why `pub mod enforce;` is intentionally absent below: declaring it
//! with nothing to back it would just fail to compile. `GatewayDaemon`
//! is generic over `Enforcer` and this crate provides no real
//! implementation outside `#[cfg(test)]`, so it cannot be constructed
//! for production use until a human's `enforce/` supplies one.

pub mod arm;
pub mod daemon;
pub mod protocol;
pub mod server;
pub mod trip_log;
pub mod types;

pub use arm::{ArmCheckResult, ArmReport, run_arm_sequence};
pub use daemon::{GatewayDaemon, GatewayError};
pub use server::GatewayServer;
pub use trip_log::{TripLog, now_unix_ms, read_entries};
pub use types::{
    BusLimits, EffectiveLimits, EnforceAction, EnforceDecision, Enforcer, FaultPolicy, JointLimits,
    TripLogEntry,
};
