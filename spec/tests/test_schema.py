"""M0 smoke test for RobotSpec. Real fixture-based checks land in M2."""
import pytest
from harness.schema import ActuatorRating, Family, RobotSpec
from pydantic import ValidationError


def _minimal_spec_dict() -> dict:
    return {
        "schema_version": 0,
        "name": "test-robot",
        "urdf_path": "robot.urdf",
        "buses": [
            {
                "name": "can0",
                "interface": "vcan",
                "bitrate": 1_000_000,
                "supply_voltage_v": 48.0,
                "supply_current_limit_a": 40.0,
            }
        ],
        "actuators": [
            {
                "name": "knee-motor",
                "rating": {
                    "family": "cubemars",
                    "model": "AK80-9",
                    "continuous_torque_nm": 9.0,
                    "peak_torque_nm": 18.0,
                    "max_velocity_rad_s": 38.2,
                    "continuous_current_a": 8.0,
                    "peak_current_a": 24.0,
                    "torque_constant_nm_per_a": 1.1,
                },
                "profile": {"family": "cubemars", "model": "AK80-9", "firmware": "1.0"},
            }
        ],
        "joints": [
            {
                "name": "knee",
                "urdf_joint": "knee_joint",
                "actuator": "knee-motor",
                "bus": "can0",
                "can_id": 1,
                "control_mode": "mit",
            }
        ],
        "poses": [{"name": "stand", "joint_positions": {"knee": 0.3}}],
    }


def test_minimal_spec_parses():
    spec = RobotSpec.model_validate(_minimal_spec_dict())
    assert spec.name == "test-robot"
    assert spec.joints[0].actuator == "knee-motor"


def test_unknown_bus_rejected():
    d = _minimal_spec_dict()
    d["joints"][0]["bus"] = "nope"
    with pytest.raises(ValidationError):
        RobotSpec.model_validate(d)


def test_unknown_actuator_rejected():
    d = _minimal_spec_dict()
    d["joints"][0]["actuator"] = "nope"
    with pytest.raises(ValidationError):
        RobotSpec.model_validate(d)


def test_duplicate_can_id_rejected():
    d = _minimal_spec_dict()
    d["joints"].append({**d["joints"][0], "name": "knee2"})
    with pytest.raises(ValidationError):
        RobotSpec.model_validate(d)


def test_peak_below_continuous_rejected():
    with pytest.raises(ValidationError):
        ActuatorRating(
            family=Family.cubemars,
            model="bad",
            continuous_torque_nm=10.0,
            peak_torque_nm=5.0,
            max_velocity_rad_s=10.0,
            continuous_current_a=5.0,
            peak_current_a=10.0,
            torque_constant_nm_per_a=1.0,
        )


def test_schema_version_wrong_rejected():
    d = _minimal_spec_dict()
    d["schema_version"] = 1
    with pytest.raises(ValidationError):
        RobotSpec.model_validate(d)
