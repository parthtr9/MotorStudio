//! Arm sequence: the pre-flight checks that must pass before the gateway
//! accepts a command stream for a joint. Per the sprint plan's M3 scope:
//! "firmware limit readback, temperature check, bus voltage check,
//! encoder range check."
//!
//! These are validation gates, not the runtime clamp/I^2t/watchdog
//! decisions `enforce/` makes on every command -- so unlike that module,
//! this one is fine to build and test now: each check is a pure function
//! over already-known values, with no family-specific protocol knowledge
//! and no safety-critical *runtime* decision (a failed arm check blocks
//! arming; it never silently alters a running command).

use crate::backend::{Backend, MotorId, ParamId};

use super::types::JointLimits;

#[derive(Debug, Clone, PartialEq)]
pub enum ArmCheckResult {
    Pass,
    Fail(String),
}

impl ArmCheckResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, ArmCheckResult::Pass)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArmReport {
    pub joint: String,
    /// (check name, result), in the order the checks ran.
    pub checks: Vec<(&'static str, ArmCheckResult)>,
}

impl ArmReport {
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|(_, r)| r.is_pass())
    }

    /// Named reasons for every failed check, for the human/trip log. Per
    /// the M3 gate test: "Arm refused with a named reason."
    pub fn failure_reasons(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter_map(|(name, r)| match r {
                ArmCheckResult::Fail(detail) => Some(format!("{name}: {detail}")),
                ArmCheckResult::Pass => None,
            })
            .collect()
    }
}

/// Per the M3 gate test: "Firmware limit looser than spec -> Arm refused
/// with a named reason." A firmware limit *tighter* than spec is fine
/// (firmware is more conservative than we are); *looser* means the
/// hardware would accept commands our software clamp doesn't expect it
/// to, so refuse arming instead of trusting it.
pub fn check_firmware_limit_not_looser_than_spec(
    firmware_limit: f64,
    spec_limit: f64,
) -> ArmCheckResult {
    if firmware_limit > spec_limit {
        ArmCheckResult::Fail(format!(
            "firmware limit {firmware_limit} is looser than spec limit {spec_limit}"
        ))
    } else {
        ArmCheckResult::Pass
    }
}

pub fn check_temperature(measured_degc: f64, max_degc: f64) -> ArmCheckResult {
    if measured_degc > max_degc {
        ArmCheckResult::Fail(format!(
            "winding temperature {measured_degc} degC exceeds arm-time max {max_degc} degC"
        ))
    } else {
        ArmCheckResult::Pass
    }
}

pub fn check_encoder_range(position_rad: f64, min_rad: f64, max_rad: f64) -> ArmCheckResult {
    if position_rad < min_rad || position_rad > max_rad {
        ArmCheckResult::Fail(format!(
            "encoder position {position_rad} rad outside spec range [{min_rad}, {max_rad}]"
        ))
    } else {
        ArmCheckResult::Pass
    }
}

pub fn check_bus_voltage(measured_v: f64, min_v: f64, max_v: f64) -> ArmCheckResult {
    if measured_v < min_v || measured_v > max_v {
        ArmCheckResult::Fail(format!(
            "bus voltage {measured_v} V outside expected range [{min_v}, {max_v}]"
        ))
    } else {
        ArmCheckResult::Pass
    }
}

/// Runs the arm sequence for one joint against a live `Backend`.
///
/// `firmware_limit_param`/`firmware_limit_field` select which firmware
/// parameter to read back and which of `limits`' fields it's compared
/// against -- this orchestrator doesn't know any family's param table, it
/// just calls `Backend::read_param` with whatever the caller (future
/// per-family arming code) supplies.
///
/// Bus voltage is not yet part of `Backend::feedback` (that's per-motor,
/// voltage is per-bus) -- callers pass it in directly for now. Revisit
/// once a bus-level feedback API exists.
pub fn run_arm_sequence(
    backend: &mut dyn Backend,
    joint: &str,
    id: MotorId,
    limits: &JointLimits,
    firmware_limit_param: Option<ParamId>,
    bus_voltage_v: Option<(f64, f64, f64)>, // (measured, min, max)
) -> ArmReport {
    let mut checks = Vec::new();

    if let Some(param) = firmware_limit_param {
        let result = match backend.read_param(id, param.clone()) {
            Ok(firmware_limit) => {
                check_firmware_limit_not_looser_than_spec(firmware_limit, limits.max_torque_nm)
            }
            Err(e) => ArmCheckResult::Fail(format!("could not read firmware limit: {e}")),
        };
        checks.push(("firmware_limit_readback", result));
    }

    let feedback_result = backend.feedback(id);
    let feedback = feedback_result.unwrap_or_default();

    if let Some(max_temp) = limits.max_winding_temp_degc {
        let result = match feedback.winding_temp_degc {
            Some(measured) => check_temperature(measured, max_temp),
            None => ArmCheckResult::Fail("no temperature feedback available".to_string()),
        };
        checks.push(("temperature", result));
    }

    let encoder_result = match feedback.position_rad {
        Some(position) => {
            check_encoder_range(position, limits.min_position_rad, limits.max_position_rad)
        }
        None => ArmCheckResult::Fail("no position feedback available".to_string()),
    };
    checks.push(("encoder_range", encoder_result));

    if let Some((measured, min_v, max_v)) = bus_voltage_v {
        checks.push(("bus_voltage", check_bus_voltage(measured, min_v, max_v)));
    }

    ArmReport {
        joint: joint.to_string(),
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Command;
    use crate::backends::mock::MockBackend;
    use crate::gateway::types::FaultPolicy;

    fn sample_limits() -> JointLimits {
        JointLimits {
            joint: "knee".to_string(),
            min_position_rad: -1.0,
            max_position_rad: 1.0,
            max_velocity_rad_s: 10.0,
            max_torque_nm: 9.0,
            continuous_current_a: 8.0,
            fault_policy: FaultPolicy::Damp,
            max_winding_temp_degc: Some(135.0),
        }
    }

    #[test]
    fn firmware_limit_looser_than_spec_fails_with_named_reason() {
        let result = check_firmware_limit_not_looser_than_spec(12.0, 9.0);
        assert!(!result.is_pass());
        match result {
            ArmCheckResult::Fail(detail) => {
                assert!(detail.contains("looser"));
            }
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn firmware_limit_tighter_than_spec_passes() {
        assert!(check_firmware_limit_not_looser_than_spec(5.0, 9.0).is_pass());
    }

    #[test]
    fn firmware_limit_equal_to_spec_passes() {
        assert!(check_firmware_limit_not_looser_than_spec(9.0, 9.0).is_pass());
    }

    #[test]
    fn temperature_over_max_fails() {
        assert!(!check_temperature(140.0, 135.0).is_pass());
    }

    #[test]
    fn temperature_under_max_passes() {
        assert!(check_temperature(60.0, 135.0).is_pass());
    }

    #[test]
    fn encoder_outside_range_fails() {
        assert!(!check_encoder_range(2.0, -1.0, 1.0).is_pass());
    }

    #[test]
    fn encoder_inside_range_passes() {
        assert!(check_encoder_range(0.5, -1.0, 1.0).is_pass());
    }

    #[test]
    fn bus_voltage_outside_range_fails() {
        assert!(!check_bus_voltage(60.0, 44.0, 52.0).is_pass());
    }

    #[test]
    fn arm_sequence_with_no_feedback_fails_rather_than_assumes_safe() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        // MockBackend::feedback returns Feedback::default() (all None),
        // never fabricated real-looking values -- so arm must fail, not
        // assume a safe default.
        let limits = sample_limits();
        let report = run_arm_sequence(&mut backend, "knee", MotorId(1), &limits, None, None);
        assert!(!report.all_passed());
        assert!(!report.failure_reasons().is_empty());
    }

    #[test]
    fn arm_sequence_reports_named_reason_for_looser_firmware_limit() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        backend
            .write_param(MotorId(1), ParamId::Name("max_torque".to_string()), 20.0)
            .unwrap();
        let limits = sample_limits();
        let report = run_arm_sequence(
            &mut backend,
            "knee",
            MotorId(1),
            &limits,
            Some(ParamId::Name("max_torque".to_string())),
            None,
        );
        let reasons = report.failure_reasons();
        assert!(
            reasons
                .iter()
                .any(|r| r.contains("firmware_limit_readback"))
        );
        assert!(reasons.iter().any(|r| r.contains("looser")));
    }

    #[test]
    fn arm_sequence_does_not_block_command_path_on_its_own() {
        // Arm checks gate arming; they don't themselves send commands.
        // Sanity check that running the sequence doesn't require (or
        // trigger) a `command` call.
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        let limits = sample_limits();
        let _ = run_arm_sequence(&mut backend, "knee", MotorId(1), &limits, None, None);
        // enable()/command() were never called, so command() must still
        // reject as "not enabled" -- confirms arm didn't silently enable.
        let result = backend.command(MotorId(1), Command::default());
        assert!(result.is_err());
    }
}
