"""Tests call the tool functions directly (the @server.tool() decorator
registers them with the MCPServer instance but returns the original
callable unchanged, so this doesn't need a running MCP session)."""
from pathlib import Path

from mcp_server.server import (
    bus_status,
    check_spec,
    motor_status,
    run_on_emulator,
    trip_log,
)

FIXTURES = Path(__file__).resolve().parents[2] / "spec" / "tests" / "fixtures"


def test_check_spec_against_known_good_fixture_is_clean():
    findings = check_spec(str(FIXTURES / "good" / "robot.yaml"))
    assert findings == []


def test_check_spec_flags_a_broken_fixture():
    findings = check_spec(str(FIXTURES / "broken" / "hold_torque_exceeded.yaml"))
    codes = [f["code"] for f in findings]
    assert "E_HOLD_TORQUE_EXCEEDED" in codes


def test_trip_log_tool_on_missing_file_returns_empty_list():
    assert trip_log("/no/such/trip.jsonl") == []


def test_bus_status_reports_not_available_rather_than_fake_data():
    result = bus_status("can0")
    assert result["available"] is False
    assert "network API" in result["reason"]


def test_motor_status_reports_not_available_rather_than_fake_data():
    result = motor_status("knee_joint")
    assert result["available"] is False


def test_run_on_emulator_reports_not_available_rather_than_fake_data():
    result = run_on_emulator("robot.yaml", "controller.py")
    assert result["available"] is False
