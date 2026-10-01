"""Shared gravity-holding-torque computation, used by both
checks.check_hold_torque (pass/fail against continuous rating) and the
MCP server's spec-time thermal_headroom tool (a continuous margin
fraction, not pass/fail). Kept in one place so the physics loop isn't
duplicated and can't drift between the two callers.
"""
from __future__ import annotations

from harness.schema import RobotSpec
from harness.urdf import Urdf, gravity_torque_nm


def required_holding_torques(spec: RobotSpec, urdf: Urdf) -> list[dict]:
    """For every (pose, joint) pair in `spec.poses`, the torque that
    joint's actuator must supply to hold statically against gravity,
    expressed at the actuator's own output (divided by
    `transmission_ratio` -- see checks.py's module docstring for the
    frame convention). Skips a joint/pose pair if its actuator or URDF
    joint can't be resolved (schema-level checks catch that
    separately); never fabricates a value for an unresolvable pair.

    Each returned dict: pose, joint, actuator, required_torque_nm,
    continuous_torque_nm.
    """
    actuators_by_name = {a.name: a for a in spec.actuators}
    joints_by_name = {j.name: j for j in spec.joints}
    results: list[dict] = []

    for pose in spec.poses:
        joint_values = {
            joints_by_name[name].urdf_joint: value
            for name, value in pose.joint_positions.items()
            if name in joints_by_name
        }
        for joint_name in pose.joint_positions:
            joint = joints_by_name.get(joint_name)
            if joint is None:
                continue
            actuator = actuators_by_name.get(joint.actuator)
            if actuator is None:
                continue
            try:
                urdf_torque = gravity_torque_nm(urdf, joint_values, joint.urdf_joint)
            except ValueError:
                continue
            required_motor_torque = abs(urdf_torque) / joint.transmission_ratio
            results.append(
                {
                    "pose": pose.name,
                    "joint": joint.name,
                    "actuator": actuator.name,
                    "required_torque_nm": required_motor_torque,
                    "continuous_torque_nm": actuator.rating.continuous_torque_nm,
                }
            )
    return results
