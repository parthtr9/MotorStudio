"""The harness MCP server. Per the sprint plan's M5 scope: bus_status,
motor_status, trip_log, thermal_headroom, check_spec, run_on_emulator.

Three of these (check_spec, trip_log, thermal_headroom) are fully real
right now -- they only need a spec file / a log file on disk, not a
running gateway. The other three (bus_status, motor_status,
run_on_emulator) need either the gateway's network API (see
driver/src/gateway/daemon.rs's module doc: not built yet) or a vcan
emulator (v1.1 per the sprint plan, not built yet). Those three are
still exposed here -- so an agent discovers the right tool names and
shapes now -- but they return a structured "not available yet" result
rather than fabricating bus/motor state. Per root CLAUDE.md's "never
guess": that applies to telemetry we don't have just as much as to
protocol bytes we haven't confirmed.
"""
from __future__ import annotations

from pathlib import Path
from typing import Any

from harness.loader import load_robot, run_default_checks
from mcp.server.mcpserver import MCPServer

from mcp_server.thermal_headroom import spec_time_thermal_headroom
from mcp_server.trip_log_reader import read_trip_log

server = MCPServer(
    name="actuator-harness",
    instructions=(
        "Spec-and-limits layer for the actuator harness. check_spec validates "
        "a robot_hw.yaml before you touch hardware. thermal_headroom estimates "
        "holding-torque margin at named poses from the spec alone (not live "
        "telemetry -- no gateway is running). bus_status/motor_status/"
        "run_on_emulator are not available yet: this project's gateway has no "
        "network API and no vcan emulator built yet, so those tools report "
        "that explicitly rather than inventing numbers."
    ),
)


def _not_available(tool: str, reason: str) -> dict[str, Any]:
    return {"available": False, "tool": tool, "reason": reason}


@server.tool()
def check_spec(spec_path: str) -> list[dict[str, Any]]:
    """Runs the harness's static checks against a robot_hw.yaml (same
    checks as `harness check`). Returns a list of findings; an empty
    list means the spec passed every default check clean.
    """
    findings = run_default_checks(spec_path)
    return [f.model_dump(mode="json") for f in findings]


@server.tool()
def thermal_headroom(spec_path: str) -> list[dict[str, Any]]:
    """Spec-time thermal headroom at every named pose in robot_hw.yaml:
    how much continuous-torque margin each joint has, computed from the
    spec's URDF mass/COM data -- NOT live telemetry (no gateway is
    running; see this module's docstring). headroom_fraction near 0 or
    negative means that pose would overheat that joint if held.
    """
    spec, urdf = load_robot(spec_path)
    return spec_time_thermal_headroom(spec, urdf)


@server.tool()
def trip_log(log_path: str, limit: int | None = 20) -> list[dict[str, Any]]:
    """Reads the gateway's trip log (JSON lines). Returns the most
    recent `limit` entries (oldest first within that window), or all
    entries if `limit` is None. An empty list means either no trips
    have happened or the log file doesn't exist yet -- both are normal,
    not errors.
    """
    return read_trip_log(Path(log_path), limit=limit)


@server.tool()
def bus_status(bus: str) -> dict[str, Any]:
    """Live CAN bus status (load, fault state) from a running gateway.
    NOT AVAILABLE YET: the gateway has no network API to query (see
    driver/src/gateway/daemon.rs's module doc) -- there is no live
    system this tool could truthfully report on.
    """
    return _not_available(
        "bus_status",
        "the gateway has no network API yet (driver/src/gateway/daemon.rs "
        "has the arm/disarm/command/watchdog state machine but no Unix-"
        "socket/WebSocket transport on top of it); nothing to query live.",
    )


@server.tool()
def motor_status(joint: str) -> dict[str, Any]:
    """Live per-motor feedback (position, velocity, torque, temperature,
    faults) from a running gateway. NOT AVAILABLE YET -- same reason as
    bus_status.
    """
    return _not_available(
        "motor_status",
        "the gateway has no network API yet; nothing to query live. See "
        "bus_status's reason for the same gap.",
    )


@server.tool()
def run_on_emulator(spec_path: str, controller_script: str) -> dict[str, Any]:
    """Runs a controller script against a vcan emulator with a thermal
    model, per the sprint plan's M5/v1.1 scope. NOT AVAILABLE YET: no
    emulator has been built (it needs a real per-family protocol
    profile to answer frames correctly, which is itself blocked on
    firmware versions and real captures -- see
    profiles/robstride/OPEN_QUESTIONS.md).
    """
    return _not_available(
        "run_on_emulator",
        "no vcan emulator exists yet; it would need a real, manual-cited "
        "protocol profile to answer frames correctly, which is still "
        "blocked (see profiles/robstride/OPEN_QUESTIONS.md).",
    )


def main() -> None:
    server.run(transport="stdio")


if __name__ == "__main__":
    main()
