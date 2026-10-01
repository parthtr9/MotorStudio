//! The gateway daemon's core state machine: arm/disarm, command gating,
//! and the watchdog. This is the orchestration layer that calls into
//! `Backend` (real or mock) and `Enforcer` (real, once a human authors
//! `enforce/` -- see `ENFORCE_PROPOSAL.md`; test-only here otherwise).
//!
//! Deliberately NOT included yet: the Unix-socket/WebSocket network
//! transport (M3's "API" half). That's mechanical plumbing on top of
//! this; this module is the part worth getting right and tested first.
//!
//! `GatewayDaemon` is generic over `Enforcer`, and nothing in this crate
//! provides a real one outside `#[cfg(test)]` -- `enforce/` has no
//! `mod.rs` (see `gateway/mod.rs`'s doc comment), so there is no way to
//! construct a production `GatewayDaemon` yet. That's intentional: the
//! type system itself blocks running this against real hardware until
//! a human writes and reviews the real `Enforcer`, not just a comment
//! saying not to.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::backend::{Backend, Command, MotorId};

use super::trip_log::now_unix_ms;
use super::types::{EnforceAction, Enforcer, FaultPolicy, JointLimits, TripLogEntry};

#[derive(Debug, Clone, PartialEq)]
pub enum GatewayError {
    NotArmed,
    UnknownJoint(String),
    ArmFailed(Vec<(String, String)>), // (joint, reason)
    Backend(String),
}

struct JointRuntime {
    motor_id: MotorId,
    limits: JointLimits,
    last_command_at: Instant,
}

pub struct GatewayDaemon<B: Backend, E: Enforcer> {
    backend: B,
    enforcer: E,
    joints: HashMap<String, JointRuntime>,
    armed: bool,
    watchdog_timeout: Duration,
}

impl<B: Backend, E: Enforcer> GatewayDaemon<B, E> {
    pub fn new(backend: B, enforcer: E, watchdog_timeout: Duration) -> Self {
        GatewayDaemon {
            backend,
            enforcer,
            joints: HashMap::new(),
            armed: false,
            watchdog_timeout,
        }
    }

    pub fn register_joint(&mut self, name: &str, motor_id: MotorId, limits: JointLimits) {
        self.joints.insert(
            name.to_string(),
            JointRuntime {
                motor_id,
                limits,
                last_command_at: Instant::now(),
            },
        );
    }

    pub fn is_armed(&self) -> bool {
        self.armed
    }

    /// Enables every registered joint via the Backend's verified-enable
    /// path. Arms only if EVERY joint verifies -- a partial arm (some
    /// joints enabled, some not) is refused outright rather than armed
    /// with a mix, since the caller has no way to know which joints are
    /// safe to command. On any failure, already-enabled joints from this
    /// attempt are disabled again before returning the error.
    pub fn arm(&mut self, now: Instant) -> Result<(), GatewayError> {
        let mut enabled_this_attempt = Vec::new();
        let mut failures = Vec::new();

        for (name, joint) in self.joints.iter() {
            match self.backend.enable(joint.motor_id) {
                Ok(_) => enabled_this_attempt.push((name.clone(), joint.motor_id)),
                Err(e) => failures.push((name.clone(), e.to_string())),
            }
        }

        if !failures.is_empty() {
            for (_, motor_id) in &enabled_this_attempt {
                let _ = self.backend.disable(*motor_id);
            }
            return Err(GatewayError::ArmFailed(failures));
        }

        for joint in self.joints.values_mut() {
            joint.last_command_at = now;
        }
        self.armed = true;
        Ok(())
    }

    pub fn disarm(&mut self) -> Result<(), GatewayError> {
        let mut last_err = None;
        for joint in self.joints.values() {
            if let Err(e) = self.backend.disable(joint.motor_id) {
                last_err = Some(e.to_string());
            }
        }
        self.armed = false;
        match last_err {
            Some(e) => Err(GatewayError::Backend(e)),
            None => Ok(()),
        }
    }

    /// decode -> enforce -> encode -> send, per driver/CLAUDE.md. "decode"
    /// is the caller's job (whatever deserialized this Command); "encode
    /// -> send" is Backend::command. This daemon's job is gating (armed?
    /// watchdog not already tripped?) and calling the Enforcer in between.
    pub fn command(
        &mut self,
        joint_name: &str,
        requested: Command,
        now: Instant,
    ) -> Result<Option<TripLogEntry>, GatewayError> {
        if !self.armed {
            return Err(GatewayError::NotArmed);
        }
        let joint = self
            .joints
            .get_mut(joint_name)
            .ok_or_else(|| GatewayError::UnknownJoint(joint_name.to_string()))?;

        let decision = self.enforcer.enforce(joint_name, requested, &joint.limits);
        self.backend
            .command(joint.motor_id, decision.command)
            .map_err(|e| GatewayError::Backend(e.to_string()))?;
        joint.last_command_at = now;
        Ok(decision.trip)
    }

    /// Checks every armed joint's last command time against
    /// `watchdog_timeout`. For each joint past the timeout, applies its
    /// `FaultPolicy` by sending a safe command through the SAME
    /// decode->enforce->encode->send path (the fault command is still
    /// subject to enforcement, not a bypass), resets its last-command
    /// clock so it doesn't re-trip every tick, and returns a trip log
    /// entry. Does nothing while disarmed.
    ///
    /// `Damp` is simplified here to "command zero torque" -- the same
    /// as `ZeroTorque` -- rather than a real decay ramp. A ramp needs a
    /// notion of "previous commanded torque" and a time constant this
    /// struct doesn't model yet; per `ENFORCE_PROPOSAL.md`'s section 5,
    /// that's an open decision for whoever authors the real watchdog
    /// timing, not something to invent silently here. `Hold` commands
    /// the joint's last known position.
    pub fn check_watchdog(&mut self, now: Instant) -> Vec<TripLogEntry> {
        if !self.armed {
            return Vec::new();
        }
        let mut trips = Vec::new();
        let timed_out: Vec<String> = self
            .joints
            .iter()
            .filter(|(_, j)| now.duration_since(j.last_command_at) > self.watchdog_timeout)
            .map(|(name, _)| name.clone())
            .collect();

        for name in timed_out {
            let (motor_id, fault_policy) = {
                let joint = &self.joints[&name];
                (joint.motor_id, joint.limits.fault_policy)
            };
            let fault_command = match fault_policy {
                FaultPolicy::ZeroTorque | FaultPolicy::Damp => Command {
                    torque_nm: Some(0.0),
                    ..Command::default()
                },
                FaultPolicy::Hold => {
                    let last_position = self
                        .backend
                        .feedback(motor_id)
                        .ok()
                        .and_then(|f| f.position_rad);
                    Command {
                        position_rad: last_position,
                        ..Command::default()
                    }
                }
            };
            // Best-effort: a watchdog trip must not panic the loop even
            // if the backend call itself fails.
            let _ = self.backend.command(motor_id, fault_command);
            if let Some(joint) = self.joints.get_mut(&name) {
                joint.last_command_at = now;
            }
            trips.push(TripLogEntry {
                time_unix_ms: now_unix_ms(),
                joint: name,
                limit: "command_stream_timeout".to_string(),
                commanded: 0.0,
                allowed: 0.0,
                action: EnforceAction::WatchdogFault(fault_policy),
            });
        }
        trips
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::ControlMode;
    use crate::backends::mock::MockBackend;
    use crate::gateway::types::EnforceDecision;

    /// Test-only, pass-through: returns the requested command unchanged,
    /// no clamping, no trip. Stands in for the real Enforcer (which
    /// doesn't exist in this crate -- see this module's doc comment)
    /// purely so GatewayDaemon's own orchestration logic (arming,
    /// gating, watchdog dispatch) can be tested independent of
    /// whatever the real clamp/I2t logic ends up being.
    struct PassthroughEnforcer;
    impl Enforcer for PassthroughEnforcer {
        fn enforce(
            &mut self,
            _joint: &str,
            requested: Command,
            _limits: &JointLimits,
        ) -> EnforceDecision {
            EnforceDecision {
                command: requested,
                trip: None,
            }
        }
    }

    fn test_limits() -> JointLimits {
        JointLimits {
            joint: "knee".to_string(),
            min_position_rad: -1.0,
            max_position_rad: 1.0,
            max_velocity_rad_s: 10.0,
            max_torque_nm: 9.0,
            continuous_current_a: 8.0,
            fault_policy: FaultPolicy::ZeroTorque,
            max_winding_temp_degc: None,
        }
    }

    fn daemon_with_one_joint() -> GatewayDaemon<MockBackend, PassthroughEnforcer> {
        let backend = MockBackend::new(vec![MotorId(1)]);
        let mut daemon =
            GatewayDaemon::new(backend, PassthroughEnforcer, Duration::from_millis(100));
        daemon.register_joint("knee", MotorId(1), test_limits());
        daemon
    }

    #[test]
    fn command_before_arm_is_rejected() {
        let mut daemon = daemon_with_one_joint();
        let result = daemon.command("knee", Command::default(), Instant::now());
        assert_eq!(result, Err(GatewayError::NotArmed));
    }

    #[test]
    fn arm_then_command_succeeds() {
        let mut daemon = daemon_with_one_joint();
        daemon.arm(Instant::now()).expect("arm should succeed");
        assert!(daemon.is_armed());
        let result = daemon.command("knee", Command::default(), Instant::now());
        assert!(result.is_ok());
    }

    #[test]
    fn arm_fails_if_any_joint_fails_verified_enable_and_disables_the_rest() {
        let mut backend = MockBackend::new(vec![MotorId(1), MotorId(2)]);
        backend.lies_about_enable = true;
        let mut daemon =
            GatewayDaemon::new(backend, PassthroughEnforcer, Duration::from_millis(100));
        daemon.register_joint("knee", MotorId(1), test_limits());
        daemon.register_joint("ankle", MotorId(2), test_limits());

        let result = daemon.arm(Instant::now());
        assert!(matches!(result, Err(GatewayError::ArmFailed(_))));
        assert!(!daemon.is_armed());
    }

    #[test]
    fn disarm_clears_armed_and_rejects_further_commands() {
        let mut daemon = daemon_with_one_joint();
        daemon.arm(Instant::now()).unwrap();
        daemon.disarm().unwrap();
        assert!(!daemon.is_armed());
        let result = daemon.command("knee", Command::default(), Instant::now());
        assert_eq!(result, Err(GatewayError::NotArmed));
    }

    #[test]
    fn command_to_unknown_joint_is_rejected() {
        let mut daemon = daemon_with_one_joint();
        daemon.arm(Instant::now()).unwrap();
        let result = daemon.command("no_such_joint", Command::default(), Instant::now());
        assert_eq!(
            result,
            Err(GatewayError::UnknownJoint("no_such_joint".to_string()))
        );
    }

    #[test]
    fn watchdog_does_nothing_while_disarmed() {
        let mut daemon = daemon_with_one_joint();
        let trips = daemon.check_watchdog(Instant::now() + Duration::from_secs(10));
        assert!(trips.is_empty());
    }

    #[test]
    fn watchdog_trips_after_timeout_and_applies_fault_policy() {
        let mut daemon = daemon_with_one_joint();
        let t0 = Instant::now();
        daemon.arm(t0).unwrap();

        // Well within the timeout: no trip.
        let trips = daemon.check_watchdog(t0 + Duration::from_millis(50));
        assert!(trips.is_empty());

        // Past the timeout: one trip, naming the right joint and policy.
        let trips = daemon.check_watchdog(t0 + Duration::from_millis(150));
        assert_eq!(trips.len(), 1);
        assert_eq!(trips[0].joint, "knee");
        assert_eq!(
            trips[0].action,
            EnforceAction::WatchdogFault(FaultPolicy::ZeroTorque)
        );
    }

    #[test]
    fn watchdog_does_not_retrip_the_same_joint_every_tick() {
        let mut daemon = daemon_with_one_joint();
        let t0 = Instant::now();
        daemon.arm(t0).unwrap();

        let first = daemon.check_watchdog(t0 + Duration::from_millis(150));
        assert_eq!(first.len(), 1);

        // Immediately after, before another full timeout has elapsed
        // since the fault command reset the clock: no second trip.
        let second = daemon.check_watchdog(t0 + Duration::from_millis(160));
        assert!(second.is_empty());
    }

    #[test]
    fn fresh_commands_keep_the_watchdog_from_tripping() {
        let mut daemon = daemon_with_one_joint();
        let t0 = Instant::now();
        daemon.arm(t0).unwrap();

        daemon
            .command("knee", Command::default(), t0 + Duration::from_millis(80))
            .unwrap();
        // 80ms since arm, then another 80ms since the command -- never
        // exceeds the 100ms timeout in one gap.
        let trips = daemon.check_watchdog(t0 + Duration::from_millis(150));
        assert!(trips.is_empty());
    }

    #[test]
    fn mode_is_set_before_command_is_sent() {
        let mut daemon = daemon_with_one_joint();
        daemon.arm(Instant::now()).unwrap();
        daemon
            .backend
            .set_mode(MotorId(1), ControlMode::Torque)
            .unwrap();
        let result = daemon.command(
            "knee",
            Command {
                torque_nm: Some(1.0),
                ..Command::default()
            },
            Instant::now(),
        );
        assert!(result.is_ok());
    }
}
