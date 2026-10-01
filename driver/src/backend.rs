//! The `Backend` trait: the only vendor seam in this codebase (see
//! `driver/CLAUDE.md`). A `Backend` impl owns one transport + one family's
//! wire protocol (motorbridge-wrapped or a from-scratch YAML-codec'd one)
//! and exposes it through this vendor-neutral interface. Nothing above
//! this trait may know a vendor-specific frame ID or byte layout.
//!
//! Units are SI everywhere at this boundary (rad, rad/s, N*m, A, V, degC,
//! s), per root `CLAUDE.md`. A `Backend` impl converts to/from vendor units
//! internally; this trait never sees them.

use std::fmt;

/// Identifies one motor on one bus. Wraps the raw CAN arbitration ID
/// component the family uses to address a motor -- NOT a CAN frame ID
/// (which may also encode a comm-type/function code alongside this).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MotorId(pub u32);

impl fmt::Display for MotorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

/// A motor found during `Backend::scan`.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredMotor {
    pub id: MotorId,
    pub family: String,
    /// Model string if the backend can determine it without the spec file
    /// (e.g. read off the motor over the wire). `None` if unknown at scan
    /// time -- never guessed.
    pub model: Option<String>,
}

/// Control mode for a `command` call. Vendor-specific mode numbers live in
/// the profile/backend, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Position,
    Velocity,
    Torque,
    /// Combined position+velocity+torque+gains command in one frame (the
    /// style the research doc calls "MIT mode"). Not every family
    /// supports this; `set_mode` returns `BackendError::NotSupported` if
    /// it doesn't.
    MixedPvt,
}

/// A command to send to one motor. All fields SI. Fields not meaningful
/// for the motor's current `ControlMode` are ignored by the backend, not
/// an error -- the gateway's `enforce` layer decides what's legal to send,
/// this struct just carries it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Command {
    pub position_rad: Option<f64>,
    pub velocity_rad_s: Option<f64>,
    pub torque_nm: Option<f64>,
    /// Position gain, for families/modes that take one in the command
    /// frame itself (e.g. mixed PVT). Unitless (vendor-defined scale);
    /// never converted, carried through as given to the human/caller.
    pub kp: Option<f64>,
    pub kd: Option<f64>,
}

/// One motor's feedback at a point in time. All fields SI.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Feedback {
    pub position_rad: Option<f64>,
    pub velocity_rad_s: Option<f64>,
    pub torque_nm: Option<f64>,
    pub winding_temp_degc: Option<f64>,
    pub fault: Option<FaultBits>,
}

/// Raw fault bits as reported by the motor. This trait does not interpret
/// bit meanings -- that mapping is family-specific and lives in the
/// profile (`profiles/<family>/*.yaml`'s fault-bit tables), not in Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FaultBits(pub u32);

impl FaultBits {
    pub fn is_clear(&self) -> bool {
        self.0 == 0
    }
}

/// How a `Backend::enable` call confirmed the motor is actually enabled.
///
/// This exists specifically because of the silent-enable failure class
/// documented in the sprint plan's research: some CubeMars firmware
/// reports MIT mode as enabled over CAN while the motor produces no
/// torque (TUM Aries PR #73). A `Backend` impl MUST pick a real
/// verification method and report it -- it must never return `confirmed:
/// true` on an ack frame alone without checking state or torque response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnableVerification {
    /// The motor's feedback frame reports an explicit enabled/running
    /// state bit, and it was observed set.
    ReportedEnabledState,
    /// The family has no reliable enabled-state bit (or it's known to lie,
    /// per the research above), so the backend instead commanded a small
    /// known torque/position and confirmed a corresponding feedback
    /// response.
    NonZeroTorqueResponse,
}

/// Result of a `Backend::enable` call.
#[derive(Debug, Clone, PartialEq)]
pub struct EnableOutcome {
    pub method: EnableVerification,
    /// Free-text detail for the trip log / human (e.g. observed values).
    pub detail: String,
}

/// A parameter table entry's identity. Families key params by a numeric
/// index (CAN-protocol tables) or sometimes by name (host-tool tables);
/// this covers both without guessing which one a given family uses where
/// -- the backend/profile decides.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamId {
    Index(u32),
    Name(String),
}

impl fmt::Display for ParamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamId::Index(i) => write!(f, "0x{:04X}", i),
            ParamId::Name(n) => write!(f, "{n}"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error("transport io error: {0}")]
    Io(String),

    #[error("timed out waiting for response from {0}")]
    Timeout(MotorId),

    #[error("protocol error for {motor}: {detail}")]
    Protocol { motor: MotorId, detail: String },

    /// Per root CLAUDE.md: "Unknown frame or param -> log + reject. Never
    /// guess." A backend MUST return this instead of best-effort decoding
    /// a frame/param it doesn't have a cited profile entry for.
    #[error("unknown frame/param from {motor}: {detail}")]
    Unknown { motor: MotorId, detail: String },

    /// `enable` could not confirm the motor is actually enabled by either
    /// `EnableVerification` method. Per CLAUDE.md, a backend must raise
    /// this rather than report success on an ack frame alone.
    #[error("enable not verified for {motor}: {detail}")]
    VerifiedEnableFailed { motor: MotorId, detail: String },

    #[error("{0} does not support this operation: {1}")]
    NotSupported(String, String),
}

/// The only vendor seam. Every backend (motorbridge-wrapped or a
/// from-scratch YAML-codec'd one) implements this and nothing above it
/// may know family-specific frame IDs or byte layouts.
pub trait Backend {
    /// Discover motors on the bus this backend owns. Must not guess a
    /// motor's family/model from an unrecognized frame -- report what was
    /// actually identified, leave `model` `None` if genuinely unknown.
    fn scan(&mut self) -> Result<Vec<DiscoveredMotor>, BackendError>;

    /// Enable a motor and verify it actually took effect per
    /// `EnableVerification`. Must return `Err(VerifiedEnableFailed)`
    /// rather than `Ok` on an unconfirmed ack.
    fn enable(&mut self, id: MotorId) -> Result<EnableOutcome, BackendError>;

    fn disable(&mut self, id: MotorId) -> Result<(), BackendError>;

    /// Zero the motor's position reference at its current physical
    /// position (not a teleport to logical zero).
    fn zero(&mut self, id: MotorId) -> Result<(), BackendError>;

    fn set_mode(&mut self, id: MotorId, mode: ControlMode) -> Result<(), BackendError>;

    /// Send one command frame. Does not enforce limits -- that's the
    /// gateway's `enforce` layer, upstream of this call.
    fn command(&mut self, id: MotorId, command: Command) -> Result<(), BackendError>;

    /// Latest feedback for a motor. M1 simplification: a poll, not a
    /// push/stream -- the gateway loop (M3) polls this at its own rate.
    /// Revisit once the gateway's async runtime is chosen.
    fn feedback(&mut self, id: MotorId) -> Result<Feedback, BackendError>;

    fn read_param(&mut self, id: MotorId, param: ParamId) -> Result<f64, BackendError>;

    fn write_param(&mut self, id: MotorId, param: ParamId, value: f64) -> Result<(), BackendError>;
}
