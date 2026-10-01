//! The gateway: sole writer to any CAN bus (per root `CLAUDE.md`), owning
//! arming, enforcement, the watchdog, and the trip log. This module ties
//! together the pieces that can be built now; the daemon loop itself
//! (WebSocket/Unix-socket API, the actual command-stream processing) is
//! M3 scope and waits on a chosen async runtime plus a real `Backend`.
//!
//! `enforce/` (the directory alongside this file) is deliberately left
//! with no `mod.rs` and is NOT declared as a module here. See
//! `ENFORCE_PROPOSAL.md` in this directory for the proposed clamp/I^2t/
//! watchdog design -- it is a proposal for a human to review and author,
//! not code this scaffolding includes. `.claude/hooks/guard.py` blocks
//! Write/Edit under `driver/src/gateway/enforce/`, so even a stub
//! `mod.rs` there has to come from a human, not an agent -- which is also
//! why `pub mod enforce;` is intentionally absent below: declaring it
//! with nothing to back it would just fail to compile.

pub mod arm;
pub mod trip_log;
pub mod types;

pub use arm::{ArmCheckResult, ArmReport, run_arm_sequence};
pub use trip_log::{TripLog, now_unix_ms};
pub use types::{
    BusLimits, EffectiveLimits, EnforceAction, EnforceDecision, Enforcer, FaultPolicy, JointLimits,
    TripLogEntry,
};
