"""Reuses the same pendulum-style correctness setup as spec/tests/
test_checks.py's hold-torque tests, since thermal_headroom shares the
exact same physics via harness.gravity.required_holding_torques.
"""
from harness.schema import RobotSpec
from harness.urdf import Urdf
from mcp_server.thermal_headroom import spec_time_thermal_headroom

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


def _spec_dict(continuous_torque_nm: float) -> dict:
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


def test_headroom_fraction_near_one_when_barely_loaded():
    required = PENDULUM_MASS_KG * GRAVITY * PENDULUM_LENGTH_M
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=required * 100))
    results = spec_time_thermal_headroom(spec, _pendulum_urdf())
    assert len(results) == 1
    assert results[0]["headroom_fraction"] > 0.98


def test_headroom_fraction_negative_when_overloaded():
    required = PENDULUM_MASS_KG * GRAVITY * PENDULUM_LENGTH_M
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=required / 2))
    results = spec_time_thermal_headroom(spec, _pendulum_urdf())
    assert results[0]["headroom_fraction"] < 0


def test_headroom_fraction_zero_at_exact_rating():
    required = PENDULUM_MASS_KG * GRAVITY * PENDULUM_LENGTH_M
    spec = RobotSpec.model_validate(_spec_dict(continuous_torque_nm=required))
    results = spec_time_thermal_headroom(spec, _pendulum_urdf())
    assert abs(results[0]["headroom_fraction"]) < 1e-3
