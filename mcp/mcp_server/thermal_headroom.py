"""Spec-time thermal headroom: how much continuous-torque margin a named
pose leaves at each joint, computed from the same gravity-torque math as
spec/harness/checks.py's check_hold_torque (via harness.gravity, shared
so the physics isn't duplicated).

This is explicitly NOT live telemetry. No gateway network API exists
yet (see driver/src/gateway/daemon.rs's module doc), so there is no
running system to read current temperature/current from. "Headroom"
here means "if you held this pose right now, how much continuous-torque
margin would you have," computed from the spec alone -- useful for an
agent to sanity-check a controller before attempting it, but not a
substitute for the real-time I^2t/thermal monitoring the gateway will
eventually provide once `enforce/` exists.
"""
from __future__ import annotations

from harness.gravity import required_holding_torques
from harness.schema import RobotSpec
from harness.urdf import Urdf


def spec_time_thermal_headroom(spec: RobotSpec, urdf: Urdf) -> list[dict]:
    """One entry per (pose, joint): headroom_fraction is
    1 - required/continuous. 1.0 means no load at all; 0.0 means
    exactly at the continuous rating; negative means over it (the same
    condition check_hold_torque's E_HOLD_TORQUE_EXCEEDED flags).
    """
    results = []
    for row in required_holding_torques(spec, urdf):
        continuous = row["continuous_torque_nm"]
        headroom_fraction = (
            1.0 - (row["required_torque_nm"] / continuous) if continuous > 0 else float("-inf")
        )
        results.append(
            {
                "pose": row["pose"],
                "joint": row["joint"],
                "required_torque_nm": row["required_torque_nm"],
                "continuous_torque_nm": continuous,
                "headroom_fraction": headroom_fraction,
            }
        )
    return results
