//! The gateway: sole writer to any CAN bus (per root `CLAUDE.md`), owning
//! arming, enforcement, the watchdog, and the trip log. `daemon` holds
//! the core state machine (arm/disarm/command-gating/watchdog); the
//! network transport on top of it (WebSocket/Unix-socket API) is not
//! built yet -- see `daemon`'s module doc.
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
pub mod trip_log;
pub mod types;

pub use arm::{ArmCheckResult, ArmReport, run_arm_sequence};
pub use daemon::{GatewayDaemon, GatewayError};
pub use trip_log::{TripLog, now_unix_ms};
pub use types::{
    BusLimits, EffectiveLimits, EnforceAction, EnforceDecision, Enforcer, FaultPolicy, JointLimits,
    TripLogEntry,
};
