"""Correctness tests for URDF FK + gravity torque, checked against the
analytical single-pendulum formula rather than any motor-specific data.

Pendulum setup: a revolute joint about world Y axis, with the child
link's center of mass offset by L along the child's local +X axis.
Rotating by theta about Y moves that point to (L*cos(theta), 0,
-L*sin(theta)) (right-hand rotation about +Y). Potential energy
U(theta) = m*g*z = -m*g*L*sin(theta), so the analytical gravity torque is
dU/dtheta = -m*g*L*cos(theta) -- maximal magnitude m*g*L at theta=0
(arm horizontal), zero at theta=+-90deg (arm vertical, equilibrium).
"""
import math

import pytest
from harness.urdf import Urdf, gravity_torque_nm

MASS_KG = 2.0
LENGTH_M = 0.5
GRAVITY = 9.80665


def _pendulum_urdf() -> Urdf:
    xml = f"""
    <robot name="pendulum_test">
      <link name="base_link"/>
      <link name="arm">
        <inertial>
          <origin xyz="{LENGTH_M} 0 0"/>
          <mass value="{MASS_KG}"/>
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


@pytest.mark.parametrize(
    "theta",
    [0.0, math.pi / 4, math.pi / 2, -math.pi / 2, math.pi / 3, 2.0],
)
def test_gravity_torque_matches_analytical_pendulum(theta: float):
    urdf = _pendulum_urdf()
    expected = -MASS_KG * GRAVITY * LENGTH_M * math.cos(theta)
    actual = gravity_torque_nm(urdf, {"pivot": theta}, "pivot", gravity_m_s2=GRAVITY)
    assert actual == pytest.approx(expected, abs=1e-3)


def test_gravity_torque_is_max_magnitude_when_horizontal():
    urdf = _pendulum_urdf()
    horizontal = abs(gravity_torque_nm(urdf, {"pivot": 0.0}, "pivot", gravity_m_s2=GRAVITY))
    vertical = abs(gravity_torque_nm(urdf, {"pivot": math.pi / 2}, "pivot", gravity_m_s2=GRAVITY))
    assert horizontal == pytest.approx(MASS_KG * GRAVITY * LENGTH_M, abs=1e-3)
    assert vertical == pytest.approx(0.0, abs=1e-3)


def test_unknown_joint_name_raises():
    urdf = _pendulum_urdf()
    with pytest.raises(ValueError):
        gravity_torque_nm(urdf, {"pivot": 0.0}, "no_such_joint")


def test_floating_joint_type_raises_not_implemented():
    xml = """
    <robot name="bad">
      <link name="base_link"/>
      <link name="free"/>
      <joint name="j" type="floating">
        <parent link="base_link"/>
        <child link="free"/>
      </joint>
    </robot>
    """
    with pytest.raises(NotImplementedError):
        Urdf.from_string(xml)


def test_massless_links_do_not_contribute_and_do_not_crash():
    xml = """
    <robot name="massless">
      <link name="base_link"/>
      <link name="floating_massless"/>
      <joint name="j" type="fixed">
        <parent link="base_link"/>
        <child link="floating_massless"/>
      </joint>
    </robot>
    """
    urdf = Urdf.from_string(xml)
    from harness.urdf import gravity_potential_energy_j

    assert gravity_potential_energy_j(urdf, {}) == 0.0


def test_multiple_root_links_rejected():
    xml = """
    <robot name="two_roots">
      <link name="base_link"/>
      <link name="orphan"/>
    </robot>
    """
    with pytest.raises(ValueError):
        Urdf.from_string(xml)


def test_joint_limit_parsed():
    urdf = _pendulum_urdf()
    joint = urdf.joints["pivot"]
    assert joint.limit_lower_rad == pytest.approx(-3.14)
    assert joint.limit_upper_rad == pytest.approx(3.14)
    assert joint.limit_velocity_rad_s == pytest.approx(10.0)
    assert joint.limit_effort_nm == pytest.approx(100.0)
