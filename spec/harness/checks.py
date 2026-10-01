"""Static checks. Pure functions, no I/O (per spec/CLAUDE.md) -- every
check takes already-loaded data and returns `Finding`s; nothing here
reads a file or hits the network. The CLI layer (`cli.py`) does I/O and
hands checks their inputs.

Frame/unit conventions assumed by these checks (our own schema's
semantics, not vendor protocol data, so stated explicitly here rather
than guessed):
- `ActuatorRating` torque/velocity fields are the actuator's own rated
  OUTPUT (post any internal gearing), matching the quasi-direct-drive
  actuators this project targets.
- `Joint.transmission_ratio` is an ADDITIONAL external transmission
  beyond the actuator's own output (default 1.0 = none, the common case
  for QDD actuators where the joint IS the actuator output shaft).
"""
from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, ValidationError

from harness.gravity import required_holding_torques
from harness.schema import RobotSpec
from harness.urdf import Urdf


class Severity(str, Enum):
    error = "error"
    warning = "warning"
    info = "info"


class Finding(BaseModel):
    code: str
    joint: str | None = None
    severity: Severity
    message: str
    fix: str


# --- schema/structural findings (from a pydantic ValidationError) ---

_MESSAGE_TO_CODE = (
    ("duplicate CAN ID", "E_DUP_CAN_ID"),
    ("unknown bus", "E_UNKNOWN_BUS"),
    ("unknown actuator", "E_UNKNOWN_ACTUATOR"),
    ("unknown joint", "E_UNKNOWN_JOINT_IN_POSE"),
    ("peak_torque_nm must be", "E_PEAK_BELOW_CONTINUOUS"),
    ("peak_current_a must be", "E_PEAK_BELOW_CONTINUOUS"),
)


def findings_from_validation_error(exc: ValidationError) -> list[Finding]:
    """Converts a RobotSpec load failure into Findings with stable codes,
    so `harness check` reports a structural problem the same way it
    reports a semantic one (code + message + fix), instead of a raw
    traceback. Pydantic v2 doesn't give our custom model_validators a
    fine-grained error type, so the code is matched from the message text
    we ourselves wrote in schema.py's validators -- not a guess about
    vendor behavior, just reading our own error strings.
    """
    findings = []
    for err in exc.errors():
        msg = err.get("msg", "")
        code = "E_SCHEMA"
        for substring, mapped_code in _MESSAGE_TO_CODE:
            if substring in msg:
                code = mapped_code
                break
        if err.get("loc") and str(err["loc"][-1]) == "schema_version":
            code = "E_SCHEMA_VERSION"
        joint = None
        loc = err.get("loc", ())
        if len(loc) >= 2 and loc[0] == "joints":
            joint = str(loc[1]) if not isinstance(loc[1], int) else None
        findings.append(
            Finding(
                code=code,
                joint=joint,
                severity=Severity.error,
                message=msg,
                fix="Fix robot_hw.yaml so it validates against the RobotSpec schema; "
                "see the field named in the error location.",
            )
        )
    return findings


# --- semantic checks ---


def check_hold_torque(spec: RobotSpec, urdf: Urdf) -> list[Finding]:
    """Flags a named pose that requires more holding torque, at any
    joint, than that joint's actuator is rated to sustain continuously --
    the "standing overheat" failure mode the sprint plan's research
    documents repeatedly (knees hitting thermal limits in <2 minutes).

    Required torque is computed via `gravity_torque_nm` (numerical
    dU/dq against the URDF's mass/COM data), divided by
    `transmission_ratio` to express it at the actuator's own output
    (see module docstring for the frame convention), then compared
    against `ActuatorRating.continuous_torque_nm`.
    """
    findings: list[Finding] = []
    for row in required_holding_torques(spec, urdf):
        if row["required_torque_nm"] > row["continuous_torque_nm"]:
            findings.append(
                Finding(
                    code="E_HOLD_TORQUE_EXCEEDED",
                    joint=row["joint"],
                    severity=Severity.error,
                    message=(
                        f"pose {row['pose']!r} requires {row['required_torque_nm']:.2f} Nm "
                        f"at {row['joint']!r}, exceeding {row['actuator']!r}'s continuous "
                        f"rating of {row['continuous_torque_nm']:.2f} Nm"
                    ),
                    fix=(
                        "Reduce the load/reach at this pose, pick a stronger actuator, "
                        "add mechanical support, or confirm (via bench test) a higher "
                        "continuous rating and mark it unverified until validated."
                    ),
                )
            )
    return findings


def check_bus_load(
    spec: RobotSpec,
    update_rate_hz: float = 1000.0,
    warn_fraction: float = 0.70,
) -> list[Finding]:
    """Estimates CAN bus utilization per bus and flags overload (>100%)
    or high load (above `warn_fraction`, a commonly recommended ceiling
    for keeping CAN latency/jitter bounded under contention -- CAN In
    Automation (CiA) guidance and common embedded-systems practice both
    suggest staying well under 100% for determinism, not a number this
    project measured itself).

    Per-frame bit count: nominal (no bit-stuffing) extended-ID (29-bit)
    CAN 2.0B frame overhead is approximated as 64 bits plus 8 data bits
    per data byte -- a standard, widely-cited approximation, NOT a
    measurement. Real buses add up to ~20% more from bit-stuffing plus
    inter-frame gaps; this is nominal, not worst-case. Each joint is
    assumed to exchange one command frame and one feedback frame per
    cycle (2 x 8-byte extended frames). `update_rate_hz` defaults to
    1000 Hz to match the gateway's own loop-rate target
    (driver/CLAUDE.md's 0.5ms p99 latency budget), since robot_hw.yaml
    has no per-bus rate field yet.

    Per the M2 gate test, this estimate is meant to be cross-checked
    against a real `canbusload` measurement (±5 percentage points) --
    that comparison needs real hardware and is not performed here.
    """
    bits_per_frame = 64 + 8 * 8  # extended-ID overhead + 8 data bytes, nominal
    frames_per_cycle_per_joint = 2  # one command + one feedback

    findings: list[Finding] = []
    joints_per_bus: dict[str, int] = {}
    for joint in spec.joints:
        joints_per_bus[joint.bus] = joints_per_bus.get(joint.bus, 0) + 1

    for bus in spec.buses:
        n_joints = joints_per_bus.get(bus.name, 0)
        bits_per_second = n_joints * frames_per_cycle_per_joint * bits_per_frame * update_rate_hz
        load_fraction = bits_per_second / bus.bitrate if bus.bitrate > 0 else float("inf")
        if load_fraction > 1.0:
            findings.append(
                Finding(
                    code="E_BUS_OVERLOAD",
                    joint=None,
                    severity=Severity.error,
                    message=(
                        f"bus {bus.name!r}: estimated load {load_fraction * 100:.0f}% "
                        f"exceeds 100% of {bus.bitrate} bps at {update_rate_hz:.0f} Hz "
                        f"with {n_joints} joints"
                    ),
                    fix="Split joints across multiple buses, lower the update rate, or "
                    "raise the bus bitrate.",
                )
            )
        elif load_fraction > warn_fraction:
            findings.append(
                Finding(
                    code="W_BUS_LOAD_HIGH",
                    joint=None,
                    severity=Severity.warning,
                    message=(
                        f"bus {bus.name!r}: estimated load {load_fraction * 100:.0f}% "
                        f"is above the recommended {warn_fraction * 100:.0f}% ceiling"
                    ),
                    fix="Consider splitting joints across buses before this bus is "
                    "measured under real load.",
                )
            )
    return findings


def check_supply_regen_budget(spec: RobotSpec) -> list[Finding]:
    """Flags a bus whose joints' combined continuous current draw
    exceeds its supply current limit, and warns when a bus has no regen
    current limit configured at all -- per the sprint plan's research,
    "undumped regen energy [is] one of the few things sure to destroy
    the FETs" (ODrive founder, ODrive#77) and misconfigured regen limits
    have fried real hardware.
    """
    findings: list[Finding] = []
    actuators_by_name = {a.name: a for a in spec.actuators}
    current_per_bus: dict[str, float] = {}
    for joint in spec.joints:
        actuator = actuators_by_name.get(joint.actuator)
        if actuator is None:
            continue
        current_per_bus[joint.bus] = (
            current_per_bus.get(joint.bus, 0.0) + actuator.rating.continuous_current_a
        )

    for bus in spec.buses:
        total_current = current_per_bus.get(bus.name, 0.0)
        if total_current > bus.supply_current_limit_a:
            findings.append(
                Finding(
                    code="E_SUPPLY_BUDGET_EXCEEDED",
                    joint=None,
                    severity=Severity.error,
                    message=(
                        f"bus {bus.name!r}: joints' combined continuous current "
                        f"{total_current:.1f} A exceeds supply limit "
                        f"{bus.supply_current_limit_a:.1f} A"
                    ),
                    fix="Raise the supply current limit/PSU capacity, or move joints "
                    "to another bus/supply.",
                )
            )
        if bus.regen_current_limit_a is None:
            findings.append(
                Finding(
                    code="W_REGEN_LIMIT_UNSET",
                    joint=None,
                    severity=Severity.warning,
                    message=(
                        f"bus {bus.name!r}: no regen_current_limit_a set -- undumped "
                        f"regen energy is a documented cause of destroyed drive FETs"
                    ),
                    fix="Set regen_current_limit_a for this bus based on the drive's "
                    "datasheet rating.",
                )
            )
    return findings


def check_unverified_ratings(spec: RobotSpec) -> list[Finding]:
    """Surfaces every ActuatorRating field marked `unverified` as a
    visible finding, so an unvalidated datasheet number doesn't sit
    silently in the spec -- it's a deliberate feature of the schema
    (M0), not a bug, but it should be loud until bench-validated.
    """
    findings: list[Finding] = []
    for actuator in spec.actuators:
        for field_name in actuator.rating.unverified:
            findings.append(
                Finding(
                    code="I_UNVERIFIED_RATING",
                    joint=None,
                    severity=Severity.info,
                    message=(
                        f"actuator {actuator.name!r}: rating field {field_name!r} "
                        f"is marked unverified"
                    ),
                    fix="Bench-validate this rating and remove it from `unverified` "
                    "once confirmed.",
                )
            )
    return findings


def check_profile_availability(
    spec: RobotSpec, available_profiles: set[tuple[str, str, str]]
) -> list[Finding]:
    """Flags an actuator whose referenced profile
    (profiles/<family>/<model>_<firmware>.yaml) doesn't exist in
    `available_profiles`. Pure function: the CLI is responsible for
    scanning the real profiles/ directory and passing in what it found,
    so this stays I/O-free. Not yet wired into `harness check`'s default
    run -- see cli.py.
    """
    findings: list[Finding] = []
    for actuator in spec.actuators:
        key = (actuator.profile.family.value, actuator.profile.model, actuator.profile.firmware)
        if key not in available_profiles:
            findings.append(
                Finding(
                    code="W_PROFILE_MISSING",
                    joint=None,
                    severity=Severity.warning,
                    message=(
                        f"actuator {actuator.name!r}: no profile found for "
                        f"{key[0]}/{key[1]}_{key[2]}"
                    ),
                    fix="Author profiles/<family>/<model>_<firmware>.yaml (see "
                    "profile-author subagent) or correct the actuator's profile reference.",
                )
            )
    return findings
