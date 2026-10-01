"""URDF import + minimal forward kinematics, used only to compute the
gravity (holding) torque at a joint pose for the hold-torque check.

Supports revolute, continuous, prismatic and fixed joints -- the types
that appear on legs/arms. floating and planar joints raise
NotImplementedError rather than being silently mis-handled (per root
CLAUDE.md's "never guess" spirit: an unsupported joint type must stop
the computation, not quietly treat it as fixed).

URDF's roll-pitch-yaw is a fixed-axis (extrinsic) XYZ Euler sequence --
this is the URDF/ROS specification's own convention (REP 103), not an
assumption made here: R = Rz(yaw) @ Ry(pitch) @ Rx(roll).
"""
from __future__ import annotations

import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

GRAVITY_M_S2 = 9.80665

_SUPPORTED_JOINT_TYPES = {"revolute", "continuous", "prismatic", "fixed"}


@dataclass
class UrdfLink:
    name: str
    mass: float = 0.0
    com_xyz: tuple[float, float, float] = (0.0, 0.0, 0.0)


@dataclass
class UrdfJoint:
    name: str
    type: str
    parent: str
    child: str
    origin_xyz: tuple[float, float, float] = (0.0, 0.0, 0.0)
    origin_rpy: tuple[float, float, float] = (0.0, 0.0, 0.0)
    axis: tuple[float, float, float] = (0.0, 0.0, 1.0)
    limit_lower_rad: float | None = None
    limit_upper_rad: float | None = None
    limit_velocity_rad_s: float | None = None
    limit_effort_nm: float | None = None


@dataclass
class Urdf:
    links: dict[str, UrdfLink] = field(default_factory=dict)
    joints: dict[str, UrdfJoint] = field(default_factory=dict)
    root_link: str = ""

    @classmethod
    def from_path(cls, path: str | Path) -> Urdf:
        return cls.from_string(Path(path).read_text())

    @classmethod
    def from_string(cls, xml_text: str) -> Urdf:
        root = ET.fromstring(xml_text)

        links: dict[str, UrdfLink] = {}
        for link_el in root.findall("link"):
            name = link_el.get("name")
            if name is None:
                raise ValueError("<link> element with no name attribute")
            mass = 0.0
            com = (0.0, 0.0, 0.0)
            inertial = link_el.find("inertial")
            if inertial is not None:
                mass_el = inertial.find("mass")
                if mass_el is not None and mass_el.get("value") is not None:
                    mass = float(mass_el.get("value"))
                origin_el = inertial.find("origin")
                if origin_el is not None:
                    com = _parse_xyz(origin_el.get("xyz"))
            links[name] = UrdfLink(name=name, mass=mass, com_xyz=com)

        joints: dict[str, UrdfJoint] = {}
        for joint_el in root.findall("joint"):
            name = joint_el.get("name")
            jtype = joint_el.get("type")
            if name is None or jtype is None:
                raise ValueError("<joint> element missing name or type attribute")
            if jtype not in _SUPPORTED_JOINT_TYPES:
                raise NotImplementedError(
                    f"joint {name!r}: type {jtype!r} is not supported by this "
                    f"FK implementation (supported: {sorted(_SUPPORTED_JOINT_TYPES)})"
                )
            parent_el = joint_el.find("parent")
            child_el = joint_el.find("child")
            if parent_el is None or child_el is None:
                raise ValueError(f"joint {name!r} missing <parent> or <child>")
            parent = parent_el.get("link")
            child = child_el.get("link")
            if parent is None or child is None:
                raise ValueError(f"joint {name!r}: <parent>/<child> missing link attribute")

            origin_xyz = (0.0, 0.0, 0.0)
            origin_rpy = (0.0, 0.0, 0.0)
            origin_el = joint_el.find("origin")
            if origin_el is not None:
                origin_xyz = _parse_xyz(origin_el.get("xyz"))
                origin_rpy = _parse_xyz(origin_el.get("rpy"))

            axis = (0.0, 0.0, 1.0)
            axis_el = joint_el.find("axis")
            if axis_el is not None and axis_el.get("xyz") is not None:
                axis = _parse_xyz(axis_el.get("xyz"))

            limit_lower = limit_upper = limit_velocity = limit_effort = None
            limit_el = joint_el.find("limit")
            if limit_el is not None:
                if limit_el.get("lower") is not None:
                    limit_lower = float(limit_el.get("lower"))
                if limit_el.get("upper") is not None:
                    limit_upper = float(limit_el.get("upper"))
                if limit_el.get("velocity") is not None:
                    limit_velocity = float(limit_el.get("velocity"))
                if limit_el.get("effort") is not None:
                    limit_effort = float(limit_el.get("effort"))

            joints[name] = UrdfJoint(
                name=name,
                type=jtype,
                parent=parent,
                child=child,
                origin_xyz=origin_xyz,
                origin_rpy=origin_rpy,
                axis=axis,
                limit_lower_rad=limit_lower,
                limit_upper_rad=limit_upper,
                limit_velocity_rad_s=limit_velocity,
                limit_effort_nm=limit_effort,
            )

        children = {j.child for j in joints.values()}
        roots = [name for name in links if name not in children]
        if len(roots) != 1:
            raise ValueError(
                f"expected exactly one root link (a link with no parent joint), "
                f"found {roots!r}"
            )
        return cls(links=links, joints=joints, root_link=roots[0])


def _parse_xyz(s: str | None) -> tuple[float, float, float]:
    if not s:
        return (0.0, 0.0, 0.0)
    parts = [float(x) for x in s.split()]
    if len(parts) != 3:
        raise ValueError(f"expected 3 space-separated floats, got {s!r}")
    return (parts[0], parts[1], parts[2])


def _rpy_matrix(rpy: tuple[float, float, float]) -> np.ndarray:
    """Fixed-axis extrinsic XYZ Euler, per the URDF spec: R = Rz(yaw) @ Ry(pitch) @ Rx(roll)."""
    roll, pitch, yaw = rpy
    cr, sr = np.cos(roll), np.sin(roll)
    cp, sp = np.cos(pitch), np.sin(pitch)
    cy, sy = np.cos(yaw), np.sin(yaw)
    rx = np.array([[1, 0, 0], [0, cr, -sr], [0, sr, cr]])
    ry = np.array([[cp, 0, sp], [0, 1, 0], [-sp, 0, cp]])
    rz = np.array([[cy, -sy, 0], [sy, cy, 0], [0, 0, 1]])
    return rz @ ry @ rx


def _axis_angle_matrix(axis: tuple[float, float, float], angle: float) -> np.ndarray:
    """Rodrigues' rotation formula for rotation by `angle` about unit `axis`."""
    axis_arr = np.array(axis, dtype=float)
    norm = float(np.linalg.norm(axis_arr))
    if norm == 0:
        raise ValueError("zero-length joint axis")
    x, y, z = axis_arr / norm
    c, s, t = np.cos(angle), np.sin(angle), 1 - np.cos(angle)
    return np.array(
        [
            [t * x * x + c, t * x * y - s * z, t * x * z + s * y],
            [t * x * y + s * z, t * y * y + c, t * y * z - s * x],
            [t * x * z - s * y, t * y * z + s * x, t * z * z + c],
        ]
    )


def _homogeneous(rotation: np.ndarray, translation: tuple[float, float, float]) -> np.ndarray:
    t = np.eye(4)
    t[:3, :3] = rotation
    t[:3, 3] = translation
    return t


def _joint_transform(joint: UrdfJoint, value: float) -> np.ndarray:
    origin = _homogeneous(_rpy_matrix(joint.origin_rpy), joint.origin_xyz)
    if joint.type == "fixed":
        motion = np.eye(4)
    elif joint.type in ("revolute", "continuous"):
        motion = _homogeneous(_axis_angle_matrix(joint.axis, value), (0.0, 0.0, 0.0))
    elif joint.type == "prismatic":
        axis_arr = np.array(joint.axis, dtype=float)
        norm = float(np.linalg.norm(axis_arr))
        if norm == 0:
            raise ValueError(f"joint {joint.name!r}: zero-length axis")
        translation = tuple(axis_arr / norm * value)
        motion = _homogeneous(np.eye(3), translation)
    else:  # pragma: no cover -- unreachable, constructor already rejects unknown types
        raise NotImplementedError(f"joint {joint.name!r}: unsupported type {joint.type!r}")
    return origin @ motion


def link_world_transforms(urdf: Urdf, joint_values: dict[str, float]) -> dict[str, np.ndarray]:
    """Forward kinematics: the world transform of every link's origin
    frame, at the given joint configuration. A joint not present in
    `joint_values` (including every `fixed` joint, which has no value)
    is treated as 0.
    """
    transforms: dict[str, np.ndarray] = {urdf.root_link: np.eye(4)}
    children_by_parent: dict[str, list[UrdfJoint]] = {}
    for joint in urdf.joints.values():
        children_by_parent.setdefault(joint.parent, []).append(joint)

    frontier = [urdf.root_link]
    while frontier:
        parent_link = frontier.pop()
        parent_t = transforms[parent_link]
        for joint in children_by_parent.get(parent_link, []):
            value = joint_values.get(joint.name, 0.0)
            transforms[joint.child] = parent_t @ _joint_transform(joint, value)
            frontier.append(joint.child)
    return transforms


def gravity_potential_energy_j(
    urdf: Urdf, joint_values: dict[str, float], gravity_m_s2: float = GRAVITY_M_S2
) -> float:
    """U(q) = sum over links of m * g * z_world(q): each link's mass
    times gravitational acceleration times its center-of-mass world-frame
    height, computed via forward kinematics. Gravity acts along -Z
    (URDF/ROS convention, REP 103), so holding a mass at higher +Z costs
    more potential energy.
    """
    transforms = link_world_transforms(urdf, joint_values)
    total = 0.0
    for link in urdf.links.values():
        if link.mass <= 0:
            continue
        t = transforms.get(link.name)
        if t is None:
            continue
        com_world = t @ np.array([*link.com_xyz, 1.0])
        total += link.mass * gravity_m_s2 * com_world[2]
    return total


def gravity_torque_nm(
    urdf: Urdf,
    joint_values: dict[str, float],
    joint_name: str,
    gravity_m_s2: float = GRAVITY_M_S2,
    eps: float = 1e-4,
) -> float:
    """Torque this joint must supply to hold statically against gravity
    at `joint_values`, via central-difference differentiation of
    potential energy: tau = dU/dq.

    Derivation: the Lagrangian equation of motion for a rigid-body chain
    is M(q) qdd + C(q, qd) qd + g(q) = tau, where g(q) = dU/dq is the
    generalized gravity force. For a static hold (qdd = qd = 0), this
    reduces to tau_required = g(q) = dU/dq. Differentiating numerically
    (rather than deriving a symbolic Jacobian per robot) generalizes to
    any URDF tree this module can parse.

    A positive/negative sign only indicates direction relative to the
    joint's URDF-defined axis; callers that compare against a torque
    *rating* should take the absolute value.
    """
    if joint_name not in urdf.joints:
        raise ValueError(f"no such URDF joint: {joint_name!r}")
    plus = dict(joint_values)
    plus[joint_name] = plus.get(joint_name, 0.0) + eps
    minus = dict(joint_values)
    minus[joint_name] = minus.get(joint_name, 0.0) - eps
    u_plus = gravity_potential_energy_j(urdf, plus, gravity_m_s2)
    u_minus = gravity_potential_energy_j(urdf, minus, gravity_m_s2)
    return (u_plus - u_minus) / (2 * eps)
