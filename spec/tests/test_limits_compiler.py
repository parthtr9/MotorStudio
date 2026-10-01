from harness.limits_compiler import UNBOUNDED_SENTINEL_RAD, compile_effective_limits
from harness.schema import RobotSpec
from harness.urdf import Urdf


def _urdf_with_limit() -> Urdf:
    xml = """
    <robot name="t">
      <link name="base_link"/>
      <link name="arm"/>
      <joint name="pivot" type="revolute">
        <parent link="base_link"/>
        <child link="arm"/>
        <axis xyz="0 1 0"/>
        <limit lower="-1.0" upper="1.0" velocity="5.0" effort="50"/>
      </joint>
    </robot>
    """
    return Urdf.from_string(xml)


def _urdf_no_limit() -> Urdf:
    xml = """
    <robot name="t">
      <link name="base_link"/>
      <link name="arm"/>
      <joint name="pivot" type="continuous">
        <parent link="base_link"/>
        <child link="arm"/>
        <axis xyz="0 1 0"/>
      </joint>
    </robot>
    """
    return Urdf.from_string(xml)


def _spec_dict(transmission_ratio: float = 1.0, user_cap_torque: float | None = None) -> dict:
    joint = {
        "name": "pivot_joint",
        "urdf_joint": "pivot",
        "actuator": "joint-motor",
        "bus": "can0",
        "can_id": 1,
        "control_mode": "mit",
        "transmission_ratio": transmission_ratio,
    }
    if user_cap_torque is not None:
        joint["limits"] = {"max_torque_nm": user_cap_torque}
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
                    "continuous_torque_nm": 9.0,
                    "peak_torque_nm": 18.0,
                    "max_velocity_rad_s": 30.0,
                    "continuous_current_a": 8.0,
                    "peak_current_a": 20.0,
                    "torque_constant_nm_per_a": 1.0,
                },
                "profile": {"family": "cubemars", "model": "X1", "firmware": "9.9"},
            }
        ],
        "joints": [joint],
    }


def test_position_bounds_come_from_urdf_limit():
    spec = RobotSpec.model_validate(_spec_dict())
    limits = compile_effective_limits(spec, _urdf_with_limit())
    j = limits["joints"][0]
    assert j["min_position_rad"] == -1.0
    assert j["max_position_rad"] == 1.0


def test_position_bounds_fall_back_to_sentinel_when_no_source():
    spec = RobotSpec.model_validate(_spec_dict())
    limits = compile_effective_limits(spec, _urdf_no_limit())
    j = limits["joints"][0]
    assert j["max_position_rad"] == UNBOUNDED_SENTINEL_RAD
    assert j["min_position_rad"] == -UNBOUNDED_SENTINEL_RAD


def test_max_torque_uses_peak_rating_not_continuous():
    spec = RobotSpec.model_validate(_spec_dict())
    limits = compile_effective_limits(spec, _urdf_with_limit())
    j = limits["joints"][0]
    assert j["max_torque_nm"] == 18.0  # peak, not the 9.0 continuous rating


def test_user_cap_tighter_than_peak_wins():
    spec = RobotSpec.model_validate(_spec_dict(user_cap_torque=5.0))
    limits = compile_effective_limits(spec, _urdf_with_limit())
    j = limits["joints"][0]
    assert j["max_torque_nm"] == 5.0


def test_transmission_ratio_scales_torque_and_velocity():
    spec = RobotSpec.model_validate(_spec_dict(transmission_ratio=2.0))
    limits = compile_effective_limits(spec, _urdf_with_limit())
    j = limits["joints"][0]
    assert j["max_torque_nm"] == 18.0 * 2.0
    # velocity candidates: actuator 30/2=15, urdf limit 5.0 -> min is 5.0
    assert j["max_velocity_rad_s"] == 5.0


def test_fault_policy_defaults_to_damp():
    spec = RobotSpec.model_validate(_spec_dict())
    limits = compile_effective_limits(spec, _urdf_with_limit())
    assert limits["joints"][0]["fault_policy"] == "damp"


def test_bus_limits_carried_through():
    spec = RobotSpec.model_validate(_spec_dict())
    limits = compile_effective_limits(spec, _urdf_with_limit())
    assert limits["buses"][0] == {
        "bus": "can0",
        "supply_current_limit_a": 40.0,
        "regen_current_limit_a": 10.0,
    }


def test_output_is_json_serializable():
    import json

    spec = RobotSpec.model_validate(_spec_dict())
    limits = compile_effective_limits(spec, _urdf_with_limit())
    json.dumps(limits)  # raises if not serializable
