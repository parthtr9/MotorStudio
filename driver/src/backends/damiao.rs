//! Backend wrapping the `motorbridge` project's Damiao vendor crate
//! (`motor_vendor_damiao`, same pinned commit as the RobStride backend
//! -- see driver/Cargo.toml and backends/robstride.rs's module doc for
//! the due-diligence this pin got before depending on it). Per the
//! sprint plan's feature evaluation, Damiao is a "Reuse" target
//! alongside RobStride.
//!
//! ## Why verified-enable matters even more here than for RobStride
//!
//! `DamiaoMotor::enable()` (their API) does not even wait for an ack --
//! it sends the enable frame and unconditionally returns `Ok(())`,
//! setting a local `disabled_hint` flag with no confirmation from the
//! motor at all. This is a stricter version of the exact silent-enable
//! failure class the sprint plan's research documents. This backend's
//! `enable()` calls their `enable()`, then actively requests fresh
//! feedback (`request_fresh_state`, which blocks until a frame newer
//! than the request arrives or times out -- not a passive poll) and
//! requires `status_code == STATUS_ENABLED` (confirmed against their
//! own `status_name` table: 0x1 = "ENABLED") before reporting success.
//!
//! ## Known limitations (documented, not hidden)
//!
//! - **No blind bus scan, and no ping either** -- unlike the RobStride
//!   vendor crate, Damiao's has no ping/discovery method at all.
//!   `scan()` uses `request_fresh_state` as a liveness probe for each
//!   motor already registered at construction time; it still can't
//!   discover an unknown motor.
//! - **`zero()` requires the motor to already be disabled** (their own
//!   `set_zero_position` enforces this via a software guard and
//!   returns an error otherwise). This backend does not silently call
//!   `disable()` first -- sequencing disable-then-zero is the caller's
//!   decision, not something to hide inside this wrapper.
//! - **Only one winding-temperature field exists on `Feedback`**, but
//!   Damiao's feedback carries two (`t_mos`, the driver board/FET
//!   temperature, and `t_rotor`, the motor/rotor temperature).
//!   `t_rotor` is used for `winding_temp_degc` since that's the one
//!   relevant to this project's thermal checks; `t_mos` is dropped at
//!   this boundary. Revisit `Feedback`'s shape if driver-board
//!   temperature turns out to matter too.
//! - **`ParamId::Name` resolves against `motor_vendor_damiao`'s own
//!   `REGISTER_TABLE`**, a separate source from this project's
//!   citation-backed profile schema, same caveat as the RobStride
//!   backend.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use motor_core::bus::{CanBus, open_can_bus};
use motor_vendor_damiao::{
    ControlMode as DmControlMode, DamiaoController, DamiaoMotor, REGISTER_TABLE, RegisterDataType,
    register_info,
};

use crate::backend::{
    Backend, BackendError, Command, ControlMode, DiscoveredMotor, EnableOutcome,
    EnableVerification, FaultBits, Feedback, MotorId, ParamId,
};

/// Per protocol.rs's `status_name`: 0x0 = DISABLED, 0x1 = ENABLED,
/// 0x8..=0xE = various faults. Cross-checked against their source, not
/// assumed.
const STATUS_ENABLED: u8 = 0x1;
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(250);

/// One motor to register with this backend, as declared in robot_hw.yaml.
pub struct DamiaoMotorConfig {
    pub motor_id: MotorId,
    pub feedback_id: u16,
    pub model: String,
}

pub struct DamiaoBackend {
    controller: DamiaoController,
    configured: HashMap<MotorId, ()>,
    last_mode: HashMap<MotorId, ControlMode>,
}

impl DamiaoBackend {
    pub fn new_socketcan(
        channel: &str,
        motors: &[DamiaoMotorConfig],
    ) -> Result<Self, BackendError> {
        let bus = open_can_bus(channel).map_err(|e| BackendError::Io(e.to_string()))?;
        Self::new(bus, motors)
    }

    pub fn new(bus: Arc<dyn CanBus>, motors: &[DamiaoMotorConfig]) -> Result<Self, BackendError> {
        let controller = DamiaoController::new(bus);
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
        Ok(DamiaoBackend {
            controller,
            configured,
            last_mode,
        })
    }

    fn motor(&self, id: MotorId) -> Result<Arc<DamiaoMotor>, BackendError> {
        let motor_id_u16 = to_u16(id)?;
        self.controller
            .get_motor(motor_id_u16)
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })
    }

    fn to_vendor_mode(mode: ControlMode) -> DmControlMode {
        match mode {
            ControlMode::Position => DmControlMode::PosVel,
            ControlMode::Velocity => DmControlMode::Vel,
            ControlMode::Torque => DmControlMode::Mit,
            ControlMode::MixedPvt => DmControlMode::Mit,
        }
    }

    fn resolve_rid(param: &ParamId) -> Option<u8> {
        match param {
            ParamId::Index(i) => u8::try_from(*i).ok(),
            ParamId::Name(name) => REGISTER_TABLE
                .iter()
                .find(|info| info.variable == name)
                .map(|info| info.rid),
        }
    }
}

fn to_u16(id: MotorId) -> Result<u16, BackendError> {
    u16::try_from(id.0).map_err(|_| BackendError::Protocol {
        motor: id,
        detail: format!("motor id {id} does not fit in Damiao's u16 device id"),
    })
}

impl Backend for DamiaoBackend {
    fn scan(&mut self) -> Result<Vec<DiscoveredMotor>, BackendError> {
        let mut found = Vec::new();
        for &id in self.configured.keys() {
            let motor = self.motor(id)?;
            if matches!(motor.request_fresh_state(DEFAULT_TIMEOUT), Ok(Some(_))) {
                found.push(DiscoveredMotor {
                    id,
                    family: "damiao".to_string(),
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
        match motor.request_fresh_state(DEFAULT_TIMEOUT) {
            Ok(Some(state)) if state.status_code == STATUS_ENABLED => Ok(EnableOutcome {
                method: EnableVerification::ReportedEnabledState,
                detail: format!(
                    "status={} ({}) after enable",
                    state.status_code, state.status_name
                ),
            }),
            Ok(Some(state)) => Err(BackendError::VerifiedEnableFailed {
                motor: id,
                detail: format!(
                    "enable sent but status={} ({}) is not ENABLED",
                    state.status_code, state.status_name
                ),
            }),
            Ok(None) => Err(BackendError::VerifiedEnableFailed {
                motor: id,
                detail: "enable sent but no feedback frame received to verify".to_string(),
            }),
            Err(e) => Err(BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
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

    /// Requires the motor to already be disabled (their own
    /// `set_zero_position` enforces this and errors otherwise) -- see
    /// module doc. Does not call `disable()` on the caller's behalf.
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
            .ensure_control_mode(Self::to_vendor_mode(mode), DEFAULT_TIMEOUT)
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
                motor.send_cmd_vel(command.velocity_rad_s.unwrap_or(0.0) as f32)
            }
            // send_cmd_pos_vel sends one frame directly -- no redundant
            // enable/mode-switch calls, unlike RobStride's convenience
            // method, so it's safe to use as-is for a per-tick command.
            ControlMode::Position => motor.send_cmd_pos_vel(
                command.position_rad.unwrap_or(0.0) as f32,
                command.velocity_rad_s.unwrap_or(0.0).abs() as f32,
            ),
        };
        result.map_err(|e| BackendError::Protocol {
            motor: id,
            detail: e.to_string(),
        })
    }

    fn feedback(&mut self, id: MotorId) -> Result<Feedback, BackendError> {
        let motor = self.motor(id)?;
        let state = motor
            .request_fresh_state(DEFAULT_TIMEOUT)
            .map_err(|e| BackendError::Protocol {
                motor: id,
                detail: e.to_string(),
            })?
            .ok_or(BackendError::Timeout(id))?;
        // Our own packing: 0 when DISABLED/ENABLED (no fault), else the
        // vendor's status_code value used directly as the bit position
        // -- documented here, not the vendor's own raw CAN bit layout
        // (motor_vendor_damiao's feedback has no bit-flag fault field
        // to begin with, just this one status_code).
        let fault_bits: u32 = if state.status_code >= 0x8 {
            1u32 << state.status_code
        } else {
            0
        };
        Ok(Feedback {
            position_rad: Some(state.pos as f64),
            velocity_rad_s: Some(state.vel as f64),
            torque_nm: Some(state.torq as f64),
            winding_temp_degc: Some(state.t_rotor as f64), // see module doc: t_mos dropped here
            fault: Some(FaultBits(fault_bits)),
        })
    }

    fn read_param(&mut self, id: MotorId, param: ParamId) -> Result<f64, BackendError> {
        let motor = self.motor(id)?;
        let rid = Self::resolve_rid(&param).ok_or_else(|| BackendError::Unknown {
            motor: id,
            detail: format!("{param} not in motor_vendor_damiao's REGISTER_TABLE"),
        })?;
        let info = register_info(rid).ok_or_else(|| BackendError::Unknown {
            motor: id,
            detail: format!("register id {rid} has no type info"),
        })?;
        let value = match info.data_type {
            RegisterDataType::Float => motor
                .get_register_f32(rid, DEFAULT_TIMEOUT)
                .map(|v| v as f64),
            RegisterDataType::UInt32 => motor
                .get_register_u32(rid, DEFAULT_TIMEOUT)
                .map(|v| v as f64),
        };
        value.map_err(|e| BackendError::Protocol {
            motor: id,
            detail: e.to_string(),
        })
    }

    fn write_param(&mut self, id: MotorId, param: ParamId, value: f64) -> Result<(), BackendError> {
        let motor = self.motor(id)?;
        let rid = Self::resolve_rid(&param).ok_or_else(|| BackendError::Unknown {
            motor: id,
            detail: format!("{param} not in motor_vendor_damiao's REGISTER_TABLE"),
        })?;
        let info = register_info(rid).ok_or_else(|| BackendError::Unknown {
            motor: id,
            detail: format!("register id {rid} has no type info"),
        })?;
        let result = match info.data_type {
            RegisterDataType::Float => motor.write_register_f32(rid, value as f32),
            RegisterDataType::UInt32 => motor.write_register_u32(rid, value as u32),
        };
        result.map_err(|e| BackendError::Protocol {
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

    const TEST_MOTOR_ID: u16 = 1;
    const TEST_FEEDBACK_ID: u16 = 0x11;
    const TEST_MODEL: &str = "4340P"; // pmax 12.5, vmax 10.0, tmax 28.0 -- see motor.rs DAMIAO_MODELS

    /// Builds a synthetic sensor-feedback frame exactly as
    /// motor_vendor_damiao's own `decode_sensor_feedback`/`accepts_frame`
    /// expect it (protocol.rs/motor.rs, read directly before writing
    /// this): byte0 low nibble = can_id, high nibble = status_code;
    /// pos/vel/torq are neutral zero (0x0000) so decode doesn't matter
    /// for the enable tests; only `status_code` is varied by the caller.
    fn status_frame(status_code: u8) -> CanFrame {
        let byte0 = (status_code << 4) | (TEST_MOTOR_ID as u8 & 0x0F);
        CanFrame {
            arbitration_id: TEST_FEEDBACK_ID as u32,
            data: [byte0, 0, 0, 0, 0, 0, 25, 30], // t_mos=25, t_rotor=30 (raw degC)
            dlc: 8,
            is_extended: false,
            is_rx: true,
        }
    }

    fn backend_with_one_motor(bus: Arc<MockBus>) -> DamiaoBackend {
        let bus_dyn: Arc<dyn CanBus> = bus;
        DamiaoBackend::new(
            bus_dyn,
            &[DamiaoMotorConfig {
                motor_id: MotorId(TEST_MOTOR_ID as u32),
                feedback_id: TEST_FEEDBACK_ID,
                model: TEST_MODEL.to_string(),
            }],
        )
        .expect("construction with a valid model must succeed")
    }

    /// Per the module doc: motor_vendor_damiao's own `enable()` sends
    /// the frame and returns `Ok(())` unconditionally, with no ack wait
    /// at all. This test proves our wrapper's `request_fresh_state` +
    /// status check actually runs and actually passes when the motor
    /// genuinely reports ENABLED (0x1).
    #[test]
    fn enable_reports_verified_when_status_enabled_observed() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus.clone());

        let bus_for_thread = bus.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            bus_for_thread.push_rx(status_frame(STATUS_ENABLED));
        });

        let outcome = backend
            .enable(MotorId(TEST_MOTOR_ID as u32))
            .expect("enable should succeed: fresh state arrives and reports ENABLED");
        assert_eq!(outcome.method, EnableVerification::ReportedEnabledState);
    }

    /// The failure-mode proof: this is the exact silent-enable class the
    /// module doc flags -- Damiao's own `enable()` has nothing to fail
    /// on (no ack at all), so if our wrapper didn't check status_code
    /// itself, a motor that never actually enabled would be reported as
    /// success. Here fresh state arrives but status_code is DISABLED
    /// (0x0), not ENABLED -- our wrapper must refuse to report success.
    #[test]
    fn enable_fails_verified_when_fresh_state_arrives_but_not_enabled() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus.clone());

        let bus_for_thread = bus.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            bus_for_thread.push_rx(status_frame(0x0)); // DISABLED, not ENABLED
        });

        let result = backend.enable(MotorId(TEST_MOTOR_ID as u32));
        match result {
            Err(BackendError::VerifiedEnableFailed { .. }) => {}
            other => panic!("expected VerifiedEnableFailed, got {other:?}"),
        }
    }

    /// When nothing responds at all, `request_fresh_state` itself times
    /// out and returns `Ok(None)` -- our wrapper surfaces that as
    /// VerifiedEnableFailed too (there's no underlying ack layer to fail
    /// first, unlike RobStride).
    #[test]
    fn enable_fails_verified_when_nothing_responds() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus);

        let result = backend.enable(MotorId(TEST_MOTOR_ID as u32));
        match result {
            Err(BackendError::VerifiedEnableFailed { .. }) => {}
            other => panic!("expected VerifiedEnableFailed, got {other:?}"),
        }
    }

    #[test]
    fn feedback_decodes_a_real_status_frame_shape() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus.clone());
        bus.push_rx(status_frame(STATUS_ENABLED));

        let feedback = backend
            .feedback(MotorId(TEST_MOTOR_ID as u32))
            .expect("feedback should decode the pushed status frame");
        // t_rotor (byte7=30) is used for winding_temp_degc; t_mos (byte6=25) is
        // dropped at this boundary -- see module doc.
        assert!((feedback.winding_temp_degc.unwrap() - 30.0).abs() < 1e-3);
        assert_eq!(feedback.fault.unwrap(), FaultBits(0)); // ENABLED (0x1) is not a fault
    }

    #[test]
    fn feedback_reports_fault_bit_for_an_over_voltage_status() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus.clone());
        bus.push_rx(status_frame(0x8)); // OVER_VOLTAGE per protocol.rs's status_name

        let feedback = backend
            .feedback(MotorId(TEST_MOTOR_ID as u32))
            .expect("feedback should decode even a fault status frame");
        assert_eq!(feedback.fault.unwrap(), FaultBits(1u32 << 0x8));
    }

    #[test]
    fn scan_omits_a_motor_that_never_responds_to_feedback_request() {
        let bus = Arc::new(MockBus::new());
        let mut backend = backend_with_one_motor(bus);
        let found = backend.scan().expect("scan itself should not error");
        assert!(
            found.is_empty(),
            "a motor that never responds to a feedback request must not be reported as discovered"
        );
    }

    #[test]
    fn motor_id_that_does_not_fit_u16_is_rejected_not_truncated() {
        let bus = Arc::new(MockBus::new());
        let bus_dyn: Arc<dyn CanBus> = bus;
        let result = DamiaoBackend::new(
            bus_dyn,
            &[DamiaoMotorConfig {
                motor_id: MotorId(100_000), // > u16::MAX
                feedback_id: TEST_FEEDBACK_ID,
                model: TEST_MODEL.to_string(),
            }],
        );
        assert!(result.is_err());
    }
}
