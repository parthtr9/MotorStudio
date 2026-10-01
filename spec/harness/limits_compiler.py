"""Effective-limit compiler: turns a validated RobotSpec + its URDF into
the `limits.json` shape the Rust gateway loads at arm time (see
driver/src/gateway/types.rs::EffectiveLimits -- the two must stay in
sync; there is no shared schema generator yet, so a field added on one
side needs the matching field added on the other by hand).

Per the architecture doc: "minimum of actuator, URDF, user cap,
transmission and profile scale." Profile scale is not yet included --
no real per-family profile exists yet (see profiles/robstride/
OPEN_QUESTIONS.md), so it's treated as 1.0 (no-op) until one does; this
is marked explicitly below, not silently omitted.

Frame/unit conventions (our own schema's semantics, stated here rather
than assumed -- see checks.py's module docstring for the same
convention applied to the hold-torque check):
- `max_torque_nm` (the hard per-command clamp) derives from the
  actuator's PEAK torque rating, since that's the instantaneous ceiling
  that must never be exceeded. Continuous-torque enforcement (I^2t) is
  a separate, time-integrated guard and uses `continuous_current_a`
  instead, already a distinct JointLimits field.
- Position/velocity bounds come from the URDF `<limit>` and/or the
  user's `JointLimitOverride`; an actuator rating has no position bound
  of its own. A joint with neither source present gets a documented
  "effectively unbounded" sentinel (not JSON `Infinity`, which isn't
  valid JSON) rather than silently passing `None` through -- see
  `UNBOUNDED_SENTINEL_RAD`.
"""
from __future__ import annotations

import math

from harness.schema import RobotSpec
from harness.urdf import Urdf

UNBOUNDED_SENTINEL_RAD = math.pi * 1000  # ~3141.6 rad; see module docstring
DEFAULT_COMMAND_RATE_LIMIT_HZ = 1000.0  # matches driver/CLAUDE.md's loop-rate target


def compile_effective_limits(
    spec: RobotSpec,
    urdf: Urdf,
    command_rate_limit_hz: float = DEFAULT_COMMAND_RATE_LIMIT_HZ,
) -> dict:
    """Returns a JSON-serializable dict matching the Rust
    `EffectiveLimits` struct exactly (field names and nesting), ready for
    `json.dump` to `limits.json`.
    """
    actuators_by_name = {a.name: a for a in spec.actuators}
    joint_limits = []

    for joint in spec.joints:
        actuator = actuators_by_name[joint.actuator]
        urdf_joint = urdf.joints.get(joint.urdf_joint)

        position_candidates_min = [-UNBOUNDED_SENTINEL_RAD]
        position_candidates_max = [UNBOUNDED_SENTINEL_RAD]
        if urdf_joint is not None:
            if urdf_joint.limit_lower_rad is not None:
                position_candidates_min.append(urdf_joint.limit_lower_rad)
            if urdf_joint.limit_upper_rad is not None:
                position_candidates_max.append(urdf_joint.limit_upper_rad)
        if joint.limits.min_position_rad is not None:
            position_candidates_min.append(joint.limits.min_position_rad)
        if joint.limits.max_position_rad is not None:
            position_candidates_max.append(joint.limits.max_position_rad)

        velocity_candidates = [actuator.rating.max_velocity_rad_s / joint.transmission_ratio]
        if urdf_joint is not None and urdf_joint.limit_velocity_rad_s is not None:
            velocity_candidates.append(urdf_joint.limit_velocity_rad_s)
        if joint.limits.max_velocity_rad_s is not None:
            velocity_candidates.append(joint.limits.max_velocity_rad_s)

        torque_candidates = [actuator.rating.peak_torque_nm * joint.transmission_ratio]
        if joint.limits.max_torque_nm is not None:
            torque_candidates.append(joint.limits.max_torque_nm)

        joint_limits.append(
            {
                "joint": joint.name,
                # tightest (largest) lower bound, tightest (smallest) upper bound
                "min_position_rad": max(position_candidates_min),
                "max_position_rad": min(position_candidates_max),
                "max_velocity_rad_s": min(velocity_candidates),
                "max_torque_nm": min(torque_candidates),
                "continuous_current_a": actuator.rating.continuous_current_a,
                "fault_policy": joint.fault_policy.value,
                "max_winding_temp_degc": actuator.rating.max_winding_temp_degc,
            }
        )

    bus_limits = [
        {
            "bus": bus.name,
            "supply_current_limit_a": bus.supply_current_limit_a,
            "regen_current_limit_a": bus.regen_current_limit_a,
        }
        for bus in spec.buses
    ]

    return {
        "schema_version": 0,
        "joints": joint_limits,
        "buses": bus_limits,
        "command_rate_limit_hz": command_rate_limit_hz,
    }
