//! Data contracts for the gateway. These types are the seam between this
//! scaffolding and `enforce/` (human-owned, see `driver/CLAUDE.md` and
//! `gateway/ENFORCE_PROPOSAL.md`): everything here is structure and
//! policy *shape*, not a safety decision itself.

use serde::{Deserialize, Serialize};

/// Deserialized `limits.json` -- the spec compiler's output (Python side,
/// `spec/harness/`), loaded by the gateway at arm time. Per the
/// architecture doc: "never hand-edited." This struct must stay in sync
/// with whatever the Python compiler actually emits; there is no schema
/// validation here yet beyond serde's own field matching, since the
/// compiler doesn't exist yet either (M2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectiveLimits {
    pub schema_version: u32,
    pub joints: Vec<JointLimits>,
    pub buses: Vec<BusLimits>,
    pub command_rate_limit_hz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JointLimits {
    pub joint: String,
    pub min_position_rad: f64,
    pub max_position_rad: f64,
    pub max_velocity_rad_s: f64,
    pub max_torque_nm: f64,
    /// For per-motor I^2t, per the sprint plan's "needs no thermal
    /// constants" framing -- this is the datasheet/profile continuous
    /// current rating, not a derived thermal model value.
    pub continuous_current_a: f64,
    pub fault_policy: FaultPolicy,
    /// Arm-time thermal ceiling, if the spec sets one. `None` means no
    /// arm-time temperature check runs for this joint -- not that the
    /// joint has no thermal limit at all (I^2t enforcement is separate
    /// and runtime, not arm-time).
    #[serde(default)]
    pub max_winding_temp_degc: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BusLimits {
    pub bus: String,
    pub supply_current_limit_a: f64,
    pub regen_current_limit_a: Option<f64>,
}

/// Per-joint behavior on a watchdog trip (command stream goes stale) or
/// an unrecoverable fault. Per the sprint plan: "Leg robots need a slow
/// sag, not a drop." The *decision* of which policy applies when lives in
/// the spec/profile; this enum is just the vocabulary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FaultPolicy {
    Damp,
    ZeroTorque,
    Hold,
}

/// What enforcement did to a single commanded value. The trip log's
/// `action` field. Variants here describe *what happened*, not *how to
/// decide it* -- the decision logic is `enforce/`'s job.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnforceAction {
    Passed,
    ClampedPosition,
    ClampedVelocity,
    ClampedTorque,
    I2tDerated,
    I2tTripped,
    BusBudgetExceeded,
    RateLimited,
    WatchdogFault(FaultPolicy),
}

/// One JSON-line entry in the trip log. Per the architecture doc: "time,
/// joint, limit, commanded, allowed, action."
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TripLogEntry {
    pub time_unix_ms: u64,
    pub joint: String,
    /// Which named limit acted, e.g. `"max_torque_nm"`. Free-text by
    /// design -- `enforce/` names it, this type doesn't enumerate every
    /// possible limit name.
    pub limit: String,
    pub commanded: f64,
    pub allowed: f64,
    pub action: EnforceAction,
}

/// The result of running one joint's command through enforcement: the
/// (possibly clamped) command to actually send, plus a trip log entry if
/// anything was clamped/tripped (`None` if the command passed through
/// unchanged).
#[derive(Debug, Clone, PartialEq)]
pub struct EnforceDecision {
    pub command: crate::backend::Command,
    pub trip: Option<TripLogEntry>,
}

/// The seam `enforce/` implements. Defined here, outside the protected
/// path, so this scaffolding can be built and tested against a stub/mock
/// implementation before `enforce/`'s real logic is authored and
/// reviewed by a human (see `ENFORCE_PROPOSAL.md`).
///
/// Per `driver/CLAUDE.md`: "Every command: decode -> enforce -> encode ->
/// send." This trait is the "enforce" step in that pipeline.
pub trait Enforcer {
    fn enforce(
        &mut self,
        joint: &str,
        requested: crate::backend::Command,
        limits: &JointLimits,
    ) -> EnforceDecision;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_limits_round_trips_through_json() {
        let limits = EffectiveLimits {
            schema_version: 0,
            joints: vec![JointLimits {
                joint: "knee".to_string(),
                min_position_rad: -1.0,
                max_position_rad: 1.0,
                max_velocity_rad_s: 10.0,
                max_torque_nm: 9.0,
                continuous_current_a: 8.0,
                fault_policy: FaultPolicy::Damp,
                max_winding_temp_degc: Some(135.0),
            }],
            buses: vec![BusLimits {
                bus: "can0".to_string(),
                supply_current_limit_a: 40.0,
                regen_current_limit_a: Some(10.0),
            }],
            command_rate_limit_hz: 1000.0,
        };
        let json = serde_json::to_string(&limits).unwrap();
        let round_tripped: EffectiveLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(limits, round_tripped);
    }

    #[test]
    fn trip_log_entry_serializes_as_one_json_object_per_line() {
        let entry = TripLogEntry {
            time_unix_ms: 1_700_000_000_000,
            joint: "knee".to_string(),
            limit: "max_torque_nm".to_string(),
            commanded: 15.0,
            allowed: 9.0,
            action: EnforceAction::ClampedTorque,
        };
        let line = serde_json::to_string(&entry).unwrap();
        assert!(!line.contains('\n'));
        let parsed: TripLogEntry = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed, entry);
    }

    #[test]
    fn watchdog_fault_action_carries_policy() {
        let entry = TripLogEntry {
            time_unix_ms: 0,
            joint: "ankle".to_string(),
            limit: "command_stream_timeout".to_string(),
            commanded: 0.0,
            allowed: 0.0,
            action: EnforceAction::WatchdogFault(FaultPolicy::Hold),
        };
        let line = serde_json::to_string(&entry).unwrap();
        let parsed: TripLogEntry = serde_json::from_str(&line).unwrap();
        assert_eq!(
            parsed.action,
            EnforceAction::WatchdogFault(FaultPolicy::Hold)
        );
    }
}
