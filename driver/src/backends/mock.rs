//! Test-only `Backend` impl with no real protocol data. Exists to exercise
//! the trait's contract -- especially verified-enable -- independent of
//! any vendor family. Configurable to reproduce the exact failure mode
//! the research doc documents (CubeMars acking MIT-mode-enabled while
//! producing no torque): set `lies_about_enable` and a real `Backend`
//! impl's `enable` must come back `Err(VerifiedEnableFailed)`, not `Ok`.

use std::collections::HashMap;

use crate::backend::{
    Backend, BackendError, Command, ControlMode, DiscoveredMotor, EnableOutcome,
    EnableVerification, Feedback, MotorId, ParamId,
};

pub struct MockBackend {
    pub motors: Vec<MotorId>,
    /// If true, `enable` simulates a motor that acks the enable command
    /// but never produces torque -- `enable` must detect this and return
    /// `VerifiedEnableFailed`, never `Ok`.
    pub lies_about_enable: bool,
    enabled: HashMap<MotorId, bool>,
    params: HashMap<(MotorId, String), f64>,
}

impl MockBackend {
    pub fn new(motors: Vec<MotorId>) -> Self {
        MockBackend {
            motors,
            lies_about_enable: false,
            enabled: HashMap::new(),
            params: HashMap::new(),
        }
    }

    fn known(&self, id: MotorId) -> Result<(), BackendError> {
        if self.motors.contains(&id) {
            Ok(())
        } else {
            Err(BackendError::Protocol {
                motor: id,
                detail: "no such motor on this mock bus".to_string(),
            })
        }
    }
}

impl Backend for MockBackend {
    fn scan(&mut self) -> Result<Vec<DiscoveredMotor>, BackendError> {
        Ok(self
            .motors
            .iter()
            .map(|&id| DiscoveredMotor {
                id,
                family: "mock".to_string(),
                model: None,
            })
            .collect())
    }

    fn enable(&mut self, id: MotorId) -> Result<EnableOutcome, BackendError> {
        self.known(id)?;
        if self.lies_about_enable {
            return Err(BackendError::VerifiedEnableFailed {
                motor: id,
                detail: "ack received but torque response stayed at zero".to_string(),
            });
        }
        self.enabled.insert(id, true);
        Ok(EnableOutcome {
            method: EnableVerification::ReportedEnabledState,
            detail: "mock motor reported enabled=true".to_string(),
        })
    }

    fn disable(&mut self, id: MotorId) -> Result<(), BackendError> {
        self.known(id)?;
        self.enabled.insert(id, false);
        Ok(())
    }

    fn zero(&mut self, id: MotorId) -> Result<(), BackendError> {
        self.known(id)?;
        Ok(())
    }

    fn set_mode(&mut self, id: MotorId, _mode: ControlMode) -> Result<(), BackendError> {
        self.known(id)?;
        Ok(())
    }

    fn command(&mut self, id: MotorId, _command: Command) -> Result<(), BackendError> {
        self.known(id)?;
        if !*self.enabled.get(&id).unwrap_or(&false) {
            return Err(BackendError::Protocol {
                motor: id,
                detail: "motor not enabled".to_string(),
            });
        }
        Ok(())
    }

    fn feedback(&mut self, id: MotorId) -> Result<Feedback, BackendError> {
        self.known(id)?;
        Ok(Feedback::default())
    }

    fn read_param(&mut self, id: MotorId, param: ParamId) -> Result<f64, BackendError> {
        self.known(id)?;
        self.params
            .get(&(id, param.to_string()))
            .copied()
            .ok_or_else(|| BackendError::Unknown {
                motor: id,
                detail: format!("param {param} not set on mock"),
            })
    }

    fn write_param(&mut self, id: MotorId, param: ParamId, value: f64) -> Result<(), BackendError> {
        self.known(id)?;
        self.params.insert((id, param.to_string()), value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_reports_configured_motors() {
        let mut backend = MockBackend::new(vec![MotorId(1), MotorId(2)]);
        let found = backend.scan().unwrap();
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn enable_succeeds_and_reports_verification_method() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        let outcome = backend.enable(MotorId(1)).unwrap();
        assert_eq!(outcome.method, EnableVerification::ReportedEnabledState);
    }

    #[test]
    fn enable_on_lying_motor_fails_verified_enable_not_ok() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        backend.lies_about_enable = true;
        let result = backend.enable(MotorId(1));
        assert!(matches!(
            result,
            Err(BackendError::VerifiedEnableFailed { .. })
        ));
    }

    #[test]
    fn command_before_enable_is_rejected() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        let result = backend.command(MotorId(1), Command::default());
        assert!(result.is_err());
    }

    #[test]
    fn command_after_enable_succeeds() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        backend.enable(MotorId(1)).unwrap();
        let result = backend.command(MotorId(1), Command::default());
        assert!(result.is_ok());
    }

    #[test]
    fn unknown_motor_is_rejected_not_guessed() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        let result = backend.enable(MotorId(99));
        assert!(result.is_err());
    }

    #[test]
    fn param_round_trip() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        backend
            .write_param(MotorId(1), ParamId::Name("loc_kp".to_string()), 42.0)
            .unwrap();
        let value = backend
            .read_param(MotorId(1), ParamId::Name("loc_kp".to_string()))
            .unwrap();
        assert_eq!(value, 42.0);
    }

    #[test]
    fn unset_param_is_unknown_not_zero_guessed() {
        let mut backend = MockBackend::new(vec![MotorId(1)]);
        let result = backend.read_param(MotorId(1), ParamId::Name("unset".to_string()));
        assert!(matches!(result, Err(BackendError::Unknown { .. })));
    }
}
