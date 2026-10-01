"""Unit tests for checks.py. Uses a fake "testfam" actuator family, like
profile_schema's tests -- no real RobStride data exists yet to test
against (see profiles/robstride/OPEN_QUESTIONS.md), and these checks are
generic across families anyway.
"""


from harness.checks import (
    Severity,
    check_bus_load,
    check_hold_torque,
    check_profile_availability,
    check_supply_regen_budget,
    check_unverified_ratings,
)
from harness.schema import RobotSpec
from harness.urdf import Urdf

PENDULUM_MASS_KG = 2.0
PENDULUM_LENGTH_M = 0.5
GRAVITY = 9.80665


def _pendulum_urdf() -> Urdf:
    xml = f"""
    <robot name="t">
      <link name="base_link"/>
      <link name="arm">
        <inertial>
          <origin xyz="{PENDULUM_LENGTH_M} 0 0"/>
          <mass value="{PENDULUM_MASS_KG}"/>
        </inertial>
      </link>
      <joint name="pivot" type="revolute">
        <parent link="base_link"/>
        <child link="arm"/>
        <axis xyz="0 1 0"/>
        <limit lower="-3.14" upper="3.14" velocity="10" effort="100"/>
      </joint>
    </robot>
    """
    return Urdf.from_string(xml)


def _spec_dict(continuous_torque_nm: float, unverified: list[str] | None = None) -> dict:
    return {
        "schema_version": 0,
        "name": "test-robot",
        "urdf_path": "unused.urdf",
        "buses": [
            {
                "name": "can0",
                "interface": "vcan",
                "bitrate": 1_000_000,
                "supply_voltage_v": 48.0,
                "supply_current_limit_a": 40.0,
                "regen_current_limit_a": 10.0,
            }
        ],
        "actuators": [
            {
                "name": "joint-motor",
                "rating": {
                    "family": "cubemars",
                    "model": "X1",
                    "continuous_torque_nm": continuous_torque_nm,
                    "peak_torque_nm": continuous_torque_nm * 2,
                    "max_velocity_rad_s": 30.0,
                    "continuous_current_a": 8.0,
                    "peak_current_a": 20.0,
                    "torque_constant_nm_per_a": 1.0,
                    "unverified": unverified or [],
                },
                "profile": {"family": "cubemars", "model": "X1", "firmware": "9.9"},
            }
        ],
        "joints": [
            {
                "name": "pivot_joint",
                "urdf_joint": "pivot",
                "actuator": "joint-motor",
                "bus": "can0",
                "can_id": 1,
                "control_mode": "mit",
            }
        ],
        "poses": [{"name": "horizontal", "joint_positions": {"pivot_joint": 0.0}}],
    }


def test_hold_torque_flags_when_exceeding_continuous_rating():
    required = PENDULUM_MASS_KG * GRAVITY * PENDULUM_LENGTH_M  # ~9.8 Nm at horizontal
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=required / 2))
    urdf = _pendulum_urdf()
    findings = check_hold_torque(spec, urdf)
    assert any(f.code == "E_HOLD_TORQUE_EXCEEDED" for f in findings)
    assert findings[0].joint == "pivot_joint"


def test_hold_torque_passes_when_within_continuous_rating():
    required = PENDULUM_MASS_KG * GRAVITY * PENDULUM_LENGTH_M
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=required * 2))
    urdf = _pendulum_urdf()
    findings = check_hold_torque(spec, urdf)
    assert findings == []


def test_bus_load_flags_overload_on_slow_bus():
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=100.0))
    findings = check_bus_load(spec, update_rate_hz=1000.0)
    # 1 joint, 1 Mbps bus, 1 kHz: well under the warn threshold.
    assert not any(f.code in ("E_BUS_OVERLOAD", "W_BUS_LOAD_HIGH") for f in findings)

    spec_dict = _spec_dict(continuous_torque_nm=100.0)
    spec_dict["buses"][0]["bitrate"] = 125_000  # legacy low-speed CAN
    slow_spec = RobotSpec.model_validate(spec_dict)
    overloaded = check_bus_load(slow_spec, update_rate_hz=1000.0)
    assert any(f.code == "E_BUS_OVERLOAD" for f in overloaded)


def test_supply_budget_flags_when_exceeded():
    spec_dict = _spec_dict(continuous_torque_nm=100.0)
    spec_dict["buses"][0]["supply_current_limit_a"] = 1.0  # less than the 8A actuator draws
    spec = RobotSpec.model_validate(spec_dict)
    findings = check_supply_regen_budget(spec)
    assert any(f.code == "E_SUPPLY_BUDGET_EXCEEDED" for f in findings)


def test_regen_unset_warns():
    spec_dict = _spec_dict(continuous_torque_nm=100.0)
    spec_dict["buses"][0]["regen_current_limit_a"] = None
    spec = RobotSpec.model_validate(spec_dict)
    findings = check_supply_regen_budget(spec)
    assert any(f.code == "W_REGEN_LIMIT_UNSET" and f.severity == Severity.warning for f in findings)


def test_regen_set_does_not_warn():
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=100.0))
    findings = check_supply_regen_budget(spec)
    assert not any(f.code == "W_REGEN_LIMIT_UNSET" for f in findings)


def test_unverified_rating_flagged_as_info():
    spec = RobotSpec.model_validate(
        _spec_dict(continuous_torque_nm=100.0, unverified=["continuous_torque_nm"])
    )
    findings = check_unverified_ratings(spec)
    assert len(findings) == 1
    assert findings[0].severity == Severity.info
    assert findings[0].code == "I_UNVERIFIED_RATING"


def test_no_unverified_fields_means_no_findings():
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=100.0))
    assert check_unverified_ratings(spec) == []


def test_profile_availability_flags_missing_profile():
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=100.0))
    findings = check_profile_availability(spec, available_profiles=set())
    assert any(f.code == "W_PROFILE_MISSING" for f in findings)


def test_profile_availability_passes_when_profile_present():
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=100.0))
    findings = check_profile_availability(
        spec, available_profiles={("cubemars", "X1", "9.9")}
    )
    assert findings == []
