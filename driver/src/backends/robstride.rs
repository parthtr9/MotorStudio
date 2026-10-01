//! Backend wrapping the `motorbridge` project's RobStride vendor crate
//! (`motor_vendor_robstride`, from https://github.com/motorbridge/motorbridge,
//! pinned at commit e8b3ac66f73a4a5a82e79c48a1707935ca4c8b96 -- see
//! driver/Cargo.toml). Per the sprint plan's feature evaluation, RobStride
//! is a "Reuse" target: motorbridge already implements the wire protocol,
//! so this backend wraps it rather than re-deriving frame encodings
//! ourselves, which would require exactly the kind of guessing root
//! CLAUDE.md forbids without a cited manual and a real capture.
//!
//! Before depending on it, its actual source was read (not assumed):
//! `motor_core` is a real, MIT-licensed library crate (has a proper
//! `[lib]` section, a clean `MotorDevice`/`CanBus` trait pair), not a
//! C-ABI/bindings-only project. Its RobStride model table's SI units
//! and torque ratings were cross-checked against this project's own
//! manual extracts: rs-01's `tmax: 17.0` matches "RobStride 01 17N.M"
//! from that manual's own title; rs-05's `tmax: 5.5` matches this
//! project's RS05 extract ("1.6N·m rated/5.5N·m peak"); the parameter
//! names `Damper`/`AddOffset` match the two fields this project's RS00/
//! RS03 extracts flagged as having inverted Max/Min in the manual
//! tables. None of this makes motorbridge's implementation a substitute
//! for a manual-cited `profiles/robstride/*.yaml` (see
//! profiles/robstride/OPEN_QUESTIONS.md) -- it's independent evidence
//! the wire-level implementation underneath this wrapper is credible,
//! not a claim that the citation-backed profile work is done.
//!
//! ## What this wrapper adds that motorbridge's own API does not
//!
//! `RobstrideMotor::enable()` (their API) only waits for a status-frame
//! ack (`send_with_status_ack`) -- it does not check whether the motor
//! actually entered run mode. This is precisely the silent-enable
//! failure class the sprint plan's research documents (CubeMars acking
//! MIT-mode-enabled while producing zero torque, TUM Aries PR #73).
//! `RobstrideBackend::enable()` calls their `enable()`, then polls
//! feedback and requires `mode_state == MODE_RUN` before reporting
//! success, returning `BackendError::VerifiedEnableFailed` otherwise.
//!
//! ## Known limitations (documented, not hidden)
//!
//! - **No blind bus scan.** `RobstrideController::add_motor` requires
//!   already knowing `motor_id`/`feedback_id`/`model` (which
//!   `robot_hw.yaml` provides). `scan()` therefore only pings motors
//!   registered at construction time; it cannot discover an unknown
//!   motor on the bus.
//! - **Position-mode commands write parameters directly** rather than
//!   calling `RobstrideMotor::send_cmd_pos_vel_csp`/`send_cmd_pos_vel_pp`:
//!   reading their source showed those convenience methods call
//!   `set_mode` + `enable` on every invocation, which would redundantly
//!   re-arm the motor on every command tick in a real-time gateway loop.
//! - **`ParamId::Name` resolves against `motor_vendor_robstride`'s own
//!   `PARAMETER_TABLE`**, not this project's citation-backed profile
//!   schema (`spec/harness/profile_schema.py`) -- those are two
//!   separate sources of parameter names until a real profile unifies
//!   them.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use motor_core::bus::{CanBus, open_can_bus};
use motor_vendor_robstride::{
    ControlMode as RsControlMode, PARAMETER_TABLE, ParameterDataType, ParameterId,
    ParameterValue as RsParameterValue, RobstrideController, RobstrideMotor, parameter_info,
};

use crate::backend::{
    Backend, BackendError, Command, ControlMode, DiscoveredMotor, EnableOutcome,
    EnableVerification, FaultBits, Feedback, MotorId, ParamId,
};

/// Per protocol.rs's `StatusFlags::mode_state` / the RS06 manual extract's
/// "bit22~23: Mode status ... 2: Motor mode [Run]" -- cross-checked, not
/// assumed from motorbridge's source alone.
const MODE_RUN: u8 = 2;
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(250);

/// One motor to register with this backend, as declared in robot_hw.yaml.
pub struct RobstrideMotorConfig {
    pub motor_id: MotorId,
    pub feedback_id: u16,
    pub model: String,
}

pub struct RobstrideBackend {
    controller: RobstrideController,
    configured: HashMap<MotorId, ()>,
    last_mode: HashMap<MotorId, ControlMode>,
}

impl RobstrideBackend {
    pub fn new_socketcan(
        channel: &str,
        motors: &[RobstrideMotorConfig],
    ) -> Result<Self, BackendError> {
        let bus = open_can_bus(channel).map_err(|e| BackendError::Io(e.to_string()))?;
        Self::new(bus, motors)
    }

    pub fn new(
        bus: Arc<dyn CanBus>,
        motors: &[RobstrideMotorConfig],
    ) -> Result<Self, BackendError> {
        let controller = RobstrideController::new(bus);
        let mut configured = HashMap::new();
        let mut last_mode = HashMap::new();
        for m in motors {
            let motor_id_u16 = to_u16(m.motor_id)?;
            controller
                .add_motor(motor_id_u16, m.feedback_id, &m.model)
                .map_err(|e| BackendError::Protocol {
                    motor: m.motor_id,
                    detail: e.to_string(),
                })?;
            configured.insert(m.motor_id, ());
            last_mode.insert(m.motor_id, ControlMode::Position);
        }
        Ok(RobstrideBackend {
            controller,
            configured,
            last_mode,
        })
    }

    fn motor(&self, id: MotorId) -> Result<Arc<RobstrideMotor>, BackendError> {
        let motor_id_u16 = to_u16(id)?;
        self.controller
            .get_motor(motor_id_u16)
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })
    }

    fn to_vendor_mode(mode: ControlMode) -> RsControlMode {
        match mode {
            ControlMode::Position => RsControlMode::PositionCsp,
            ControlMode::Velocity => RsControlMode::Velocity,
            ControlMode::Torque => RsControlMode::Mit,
            ControlMode::MixedPvt => RsControlMode::Mit,
        }
    }

    fn resolve_param_id(param: &ParamId) -> Option<u16> {
        match param {
            ParamId::Index(i) => u16::try_from(*i).ok(),
            ParamId::Name(name) => PARAMETER_TABLE
                .iter()
                .find(|info| info.name == name)
                .map(|info| info.id),
        }
    }

    fn value_to_f64(value: RsParameterValue) -> f64 {
        match value {
            RsParameterValue::I8(v) => v as f64,
            RsParameterValue::U8(v) => v as f64,
            RsParameterValue::U16(v) => v as f64,
            RsParameterValue::U32(v) => v as f64,
            RsParameterValue::F32(v) => v as f64,
        }
    }
}

fn to_u16(id: MotorId) -> Result<u16, BackendError> {
    u16::try_from(id.0).map_err(|_| BackendError::Protocol {
        motor: id,
        detail: format!("motor id {id} does not fit in RobStride's u16 device id"),
    })
}

impl Backend for RobstrideBackend {
    fn scan(&mut self) -> Result<Vec<DiscoveredMotor>, BackendError> {
        let mut found = Vec::new();
        for &id in self.configured.keys() {
            let motor = self.motor(id)?;
            if motor.ping(DEFAULT_TIMEOUT).is_ok() {
                found.push(DiscoveredMotor {
                    id,
                    family: "robstride".to_string(),
                    model: Some(motor.model.clone()),
                });
            }
        }
        Ok(found)
    }

    fn enable(&mut self, id: MotorId) -> Result<EnableOutcome, BackendError> {
        let motor = self.motor(id)?;
        motor.enable().map_err(|e| BackendError::Protocol {
            motor: id,
            detail: e.to_string(),
        })?;
        self.controller
            .poll_feedback_once()
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })?;
        match motor.latest_state() {
            Some(state) if state.mode_state == MODE_RUN => Ok(EnableOutcome {
                method: EnableVerification::ReportedEnabledState,
                detail: format!(
                    "mode_state={} (Run) observed after enable",
                    state.mode_state
                ),
            }),
            Some(state) => Err(BackendError::VerifiedEnableFailed {
                motor: id,
                detail: format!(
                    "enable ack'd but mode_state={} is not Run ({MODE_RUN})",
                    state.mode_state
                ),
            }),
            None => Err(BackendError::VerifiedEnableFailed {
                motor: id,
                detail: "enable ack'd but no feedback frame received to verify".to_string(),
            }),
        }
    }

    fn disable(&mut self, id: MotorId) -> Result<(), BackendError> {
        self.motor(id)?
            .disable()
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })
    }

    fn zero(&mut self, id: MotorId) -> Result<(), BackendError> {
        self.motor(id)?
            .set_zero_position()
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })
    }

    fn set_mode(&mut self, id: MotorId, mode: ControlMode) -> Result<(), BackendError> {
        let motor = self.motor(id)?;
        motor
            .set_mode(Self::to_vendor_mode(mode))
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })?;
        self.last_mode.insert(id, mode);
        Ok(())
    }

    fn command(&mut self, id: MotorId, command: Command) -> Result<(), BackendError> {
        let motor = self.motor(id)?;
        let mode = *self.last_mode.get(&id).unwrap_or(&ControlMode::Position);
        let result = match mode {
            ControlMode::MixedPvt => motor.send_cmd_mit(
                command.position_rad.unwrap_or(0.0) as f32,
                command.velocity_rad_s.unwrap_or(0.0) as f32,
                command.kp.unwrap_or(0.0) as f32,
                command.kd.unwrap_or(0.0) as f32,
                command.torque_nm.unwrap_or(0.0) as f32,
            ),
            ControlMode::Torque => {
                motor.send_cmd_mit(0.0, 0.0, 0.0, 0.0, command.torque_nm.unwrap_or(0.0) as f32)
            }
            ControlMode::Velocity => {
                motor.set_velocity_target(command.velocity_rad_s.unwrap_or(0.0) as f32)
            }
            // Direct parameter writes, not send_cmd_pos_vel_csp -- see
            // module doc. No `?` here: that would propagate through
            // `command()`'s own BackendError return type rather than
            // producing the motor_core::Result<()> every arm must share,
            // so the velocity-limit write is chained via `and_then`.
            ControlMode::Position => {
                let velocity_limit_result = match command.velocity_rad_s {
                    Some(v) => motor.write_parameter(
                        ParameterId::VelocityLimit as u16,
                        RsParameterValue::F32(v.abs() as f32),
                    ),
                    None => Ok(()),
                };
                velocity_limit_result.and_then(|_| {
                    motor.write_parameter(
                        ParameterId::PositionTarget as u16,
                        RsParameterValue::F32(command.position_rad.unwrap_or(0.0) as f32),
                    )
                })
            }
        };
        result.map_err(|e| BackendError::Protocol {
            motor: id,
            detail: e.to_string(),
        })
    }

    fn feedback(&mut self, id: MotorId) -> Result<Feedback, BackendError> {
        let motor = self.motor(id)?;
        self.controller
            .poll_feedback_once()
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })?;
        let state = motor.latest_state().ok_or(BackendError::Timeout(id))?;
        // Our own bit packing of motorbridge's already-decoded fault
        // booleans, not the vendor's original raw bit positions (those
        // were consumed by motor_vendor_robstride's own decoding).
        let fault_bits: u32 = (state.uncalibrated as u32)
            | ((state.stall as u32) << 1)
            | ((state.magnetic_encoder_fault as u32) << 2)
            | ((state.overtemperature as u32) << 3)
            | ((state.overcurrent as u32) << 4)
            | ((state.undervoltage as u32) << 5);
        Ok(Feedback {
            position_rad: Some(state.position as f64),
            velocity_rad_s: Some(state.velocity as f64),
            torque_nm: Some(state.torque as f64),
            winding_temp_degc: Some(state.temperature_c as f64),
            fault: Some(FaultBits(fault_bits)),
        })
    }

    fn read_param(&mut self, id: MotorId, param: ParamId) -> Result<f64, BackendError> {
        let motor = self.motor(id)?;
        let param_id = Self::resolve_param_id(&param).ok_or_else(|| BackendError::Unknown {
            motor: id,
            detail: format!("{param} not in motor_vendor_robstride's PARAMETER_TABLE"),
        })?;
        let value = motor
            .get_parameter(param_id, DEFAULT_TIMEOUT)
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })?;
        Ok(Self::value_to_f64(value))
    }

    fn write_param(&mut self, id: MotorId, param: ParamId, value: f64) -> Result<(), BackendError> {
        let motor = self.motor(id)?;
        let param_id = Self::resolve_param_id(&param).ok_or_else(|| BackendError::Unknown {
            motor: id,
            detail: format!("{param} not in motor_vendor_robstride's PARAMETER_TABLE"),
        })?;
        let info = parameter_info(param_id).ok_or_else(|| BackendError::Unknown {
            motor: id,
            detail: format!("param id 0x{param_id:04X} has no type info"),
        })?;
        let rs_value = match info.data_type {
            ParameterDataType::Int8 => RsParameterValue::I8(value as i8),
            ParameterDataType::UInt8 => RsParameterValue::U8(value as u8),
            ParameterDataType::UInt16 => RsParameterValue::U16(value as u16),
            ParameterDataType::UInt32 => RsParameterValue::U32(value as u32),
            ParameterDataType::Float32 => RsParameterValue::F32(value as f32),
        };
        motor
            .write_parameter(param_id, rs_value)
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motor_core::bus::CanFrame;
    use motor_core::test_support::MockBus;
    use motor_vendor_robstride::protocol::{CommunicationType, build_ext_id};

    const TEST_MOTOR_ID: u16 = 1;
    const TEST_FEEDBACK_ID: u16 = 0xFD;

    /// Builds a synthetic OPERATION_STATUS frame exactly as
    /// motor_vendor_robstride's own `decode_status_frame`/`accepts_frame`
    /// expect it (see protocol.rs/motor.rs, read directly before writing
    /// this): mode_state in extra_data bits 14-15, device_id in extra_data
    /// bits 0-7, position/velocity/torque/temperature as big-endian u16
    /// scaled against (p/v/t)max. All physical fields neutral (midpoint,
    /// 25.0 degC) except mode_state, which the caller sets.
    fn status_frame(device_id: u8, mode_state: u8) -> CanFrame {
        let extra_data: u16 = ((mode_state as u16) << 14) | (device_id as u16 & 0xFF);
        let neutral = 0x7FFFu16; // (0x7FFF/0x7FFF - 1) * max == 0.0 for pos/vel/torque
        let mut data = [0u8; 8];
        data[0..2].copy_from_slice(&neutral.to_be_bytes());
        data[2..4].copy_from_slice(&neutral.to_be_bytes());
        data[4..6].copy_from_slice(&neutral.to_be_bytes());
        data[6..8].copy_from_slice(&250u16.to_be_bytes()); // 25.0 degC (*0.1 scale)
        CanFrame {
            arbitration_id: build_ext_id(
                CommunicationType::OPERATION_STATUS,
                extra_data,
                TEST_FEEDBACK_ID as u8,
            ),
            data,
            dlc: 8,
            is_extended: true,
            is_rx: true,
        }
    }

    fn backend_with_one_motor(bus: Arc<MockBus>) -> RobstrideBackend {
        let bus_dyn: Arc<dyn CanBus> = bus;
        RobstrideBackend::new(
            bus_dyn,
            &[RobstrideMotorConfig {
                motor_id: MotorId(TEST_MOTOR_ID as u32),
                feedback_id: TEST_FEEDBACK_ID,
                model: "rs-01".to_string(),
            }],
        )
        .expect("construction with a valid model must succeed")
    }

    /// The core thing this wrapper exists for: motor_vendor_robstride's
    /// own `enable()` only waits for ANY status-frame ack (status_seq
    /// incrementing) -- it does not check mode_state. This test proves
    /// our wrapper's additional check actually runs and actually passes
    /// when the motor genuinely reports Run.
    #[test]
    fn enable_reports_verified_when_mode_run_observed() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus.clone());

        // Simulate the motor's response arriving shortly after enable()
        // sends its ENABLE frame -- background polling (started
        // automatically when the motor was added) drains it as soon as
        // it lands, same as it would from a real bus.
        let bus_for_thread = bus.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            bus_for_thread.push_rx(status_frame(TEST_MOTOR_ID as u8, MODE_RUN));
        });

        let outcome = backend
            .enable(MotorId(TEST_MOTOR_ID as u32))
            .expect("enable should succeed: ack arrives and reports Run");
        assert_eq!(outcome.method, EnableVerification::ReportedEnabledState);
    }

    /// The failure-mode proof: the underlying ack succeeds (any status
    /// frame satisfies motor_vendor_robstride's own wait), but
    /// mode_state is NOT Run -- our wrapper must still refuse to report
    /// success. This is the exact CubeMars silent-enable bug class
    /// (TUM Aries PR #73) this project's research flagged, reproduced
    /// here against RobStride's real wire shape instead of a mock
    /// backend's hypothetical one.
    #[test]
    fn enable_fails_verified_when_ack_arrives_but_mode_is_not_run() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus.clone());

        let bus_for_thread = bus.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            bus_for_thread.push_rx(status_frame(TEST_MOTOR_ID as u8, 0)); // 0 = Reset, not Run
        });

        let result = backend.enable(MotorId(TEST_MOTOR_ID as u32));
        match result {
            Err(BackendError::VerifiedEnableFailed { .. }) => {}
            other => panic!("expected VerifiedEnableFailed, got {other:?}"),
        }
    }

    /// When nothing responds at all, motor_vendor_robstride's own ack
    /// wait times out first -- our wrapper surfaces that as a Protocol
    /// error (it never reaches our own verification step, since there
    /// was no successful ack to verify on top of).
    #[test]
    fn enable_times_out_through_underlying_ack_when_nothing_responds() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus);

        let result = backend.enable(MotorId(TEST_MOTOR_ID as u32));
        match result {
            Err(BackendError::Protocol { .. }) => {}
            other => panic!("expected Protocol (underlying ack timeout), got {other:?}"),
        }
    }

    #[test]
    fn feedback_decodes_a_real_status_frame_shape() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus.clone());
        bus.push_rx(status_frame(TEST_MOTOR_ID as u8, MODE_RUN));
        std::thread::sleep(Duration::from_millis(10));

        let feedback = backend
            .feedback(MotorId(TEST_MOTOR_ID as u32))
            .expect("feedback should decode the pushed status frame");
        assert!((feedback.position_rad.unwrap()).abs() < 1e-3);
        assert!((feedback.velocity_rad_s.unwrap()).abs() < 1e-3);
        assert!((feedback.torque_nm.unwrap()).abs() < 1e-3);
        assert!((feedback.winding_temp_degc.unwrap() - 25.0).abs() < 1e-3);
        assert_eq!(feedback.fault.unwrap(), FaultBits(0));
    }

    #[test]
    fn scan_omits_a_motor_that_never_responds_to_ping() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus);
        let found = backend.scan().expect("scan itself should not error");
        assert!(
            found.is_empty(),
            "a motor that never responds to ping must not be reported as discovered"
        );
    }

    #[test]
    fn motor_id_that_does_not_fit_u16_is_rejected_not_truncated() {
        let bus = Arc::new(MockBus::new());
        let bus_dyn: Arc<dyn CanBus> = bus;
        let result = RobstrideBackend::new(
            bus_dyn,
            &[RobstrideMotorConfig {
                motor_id: MotorId(100_000), // > u16::MAX
                feedback_id: TEST_FEEDBACK_ID,
                model: "rs-01".to_string(),
            }],
        );
        assert!(result.is_err());
    }
}
