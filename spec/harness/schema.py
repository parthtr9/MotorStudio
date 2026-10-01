"""Spec schema v0. The human-written robot_hw.yaml is validated against RobotSpec.

See root CLAUDE.md: units SI everywhere (rad, rad/s, Nm, A, V, degC, s).
"""
from __future__ import annotations

from enum import Enum
from typing import Literal

from pydantic import BaseModel, Field, model_validator

SCHEMA_VERSION = 0


class Family(str, Enum):
    cubemars = "cubemars"
    robstride = "robstride"
    damiao = "damiao"
    myactuator = "myactuator"
    hexfellow = "hexfellow"
    hightorque = "hightorque"


class ControlMode(str, Enum):
    mit = "mit"
    servo = "servo"
    position = "position"
    velocity = "velocity"
    torque = "torque"


class ActuatorRating(BaseModel):
    """Datasheet ratings for one motor model. Fields not confirmed against a
    manual or bench measurement must be listed in `unverified`.
    """

    model_config = {"extra": "forbid"}

    family: Family
    model: str
    continuous_torque_nm: float = Field(gt=0)
    peak_torque_nm: float = Field(gt=0)
    max_velocity_rad_s: float = Field(gt=0)
    continuous_current_a: float = Field(gt=0)
    peak_current_a: float = Field(gt=0)
    torque_constant_nm_per_a: float = Field(gt=0)
    thermal_time_constant_s: float | None = Field(default=None, gt=0)
    max_winding_temp_degc: float | None = None
    gear_ratio: float = Field(default=1.0, gt=0)
    unverified: list[str] = Field(default_factory=list)

    @model_validator(mode="after")
    def _peak_ge_continuous(self) -> ActuatorRating:
        if self.peak_torque_nm < self.continuous_torque_nm:
            raise ValueError("peak_torque_nm must be >= continuous_torque_nm")
        if self.peak_current_a < self.continuous_current_a:
            raise ValueError("peak_current_a must be >= continuous_current_a")
        return self


class ProfileRef(BaseModel):
    """Points at profiles/<family>/<model>_<fw>.yaml."""

    model_config = {"extra": "forbid"}

    family: Family
    model: str
    firmware: str


class JointLimitOverride(BaseModel):
    """User-specified cap. Effective limit compiler takes the min of this,
    the actuator rating, the URDF joint limit, and the profile scale.
    """

    model_config = {"extra": "forbid"}

    max_torque_nm: float | None = Field(default=None, gt=0)
    max_velocity_rad_s: float | None = Field(default=None, gt=0)
    max_position_rad: float | None = None
    min_position_rad: float | None = None


class Actuator(BaseModel):
    model_config = {"extra": "forbid"}

    name: str
    rating: ActuatorRating
    profile: ProfileRef


class Joint(BaseModel):
    """Maps a URDF joint name to an actuator, bus and CAN ID, with optional
    user-specified caps tighter than the actuator rating.
    """

    model_config = {"extra": "forbid"}

    name: str
    urdf_joint: str
    actuator: str  # Actuator.name
    bus: str
    can_id: int = Field(ge=0, le=0x1FFFFFFF)
    control_mode: ControlMode
    transmission_ratio: float = Field(default=1.0, gt=0)
    limits: JointLimitOverride = Field(default_factory=JointLimitOverride)


class Pose(BaseModel):
    """Named pose for the hold-torque check: joint name -> position (rad)."""

    model_config = {"extra": "forbid"}

    name: str
    joint_positions: dict[str, float]


class BusConfig(BaseModel):
    model_config = {"extra": "forbid"}

    name: str
    interface: Literal["socketcan", "slcan", "pcan", "gs_usb", "replay", "vcan"]
    bitrate: int = Field(default=1_000_000, gt=0)
    supply_voltage_v: float = Field(gt=0)
    supply_current_limit_a: float = Field(gt=0)
    regen_current_limit_a: float | None = Field(default=None, gt=0)


class RobotSpec(BaseModel):
    """Root of robot_hw.yaml."""

    model_config = {"extra": "forbid"}

    schema_version: Literal[0] = 0
    name: str
    urdf_path: str
    buses: list[BusConfig]
    actuators: list[Actuator]
    joints: list[Joint]
    poses: list[Pose] = Field(default_factory=list)

    @model_validator(mode="after")
    def _refs_resolve(self) -> RobotSpec:
        bus_names = {b.name for b in self.buses}
        actuator_names = {a.name for a in self.actuators}
        for j in self.joints:
            if j.bus not in bus_names:
                raise ValueError(f"joint {j.name}: unknown bus {j.bus!r}")
            if j.actuator not in actuator_names:
                raise ValueError(f"joint {j.name}: unknown actuator {j.actuator!r}")
        joint_names = {j.name for j in self.joints}
        for p in self.poses:
            for jn in p.joint_positions:
                if jn not in joint_names:
                    raise ValueError(f"pose {p.name}: unknown joint {jn!r}")
        ids_per_bus: dict[str, set[int]] = {}
        for j in self.joints:
            seen = ids_per_bus.setdefault(j.bus, set())
            if j.can_id in seen:
                raise ValueError(f"bus {j.bus}: duplicate CAN ID {j.can_id}")
            seen.add(j.can_id)
        return self
