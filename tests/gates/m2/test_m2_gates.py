"""M2 gate tests.

Per the sprint plan, M2's gates are:

  1. Suite of (ideally 20) deliberately broken configs -> 100% flagged,
     each with the right error code.
  2. Known-good config -> 0 findings.
  3. Hold torque at standing pose vs measured current x Kt on the robot
     -> within +-10% on every loaded joint.
  4. Bus load estimate vs measured `canbusload` -> within +-5
     percentage points.

Gates 3 and 4 need real hardware (a bench measurement, a real CAN
capture under `canbusload`) and are marked NEEDS-HUMAN below -- they
can't be proven by this test suite alone. Gates 1 and 2 are fully
automated here.

This suite currently ships 11 broken fixtures (one per implemented error
code), not the full 20 the sprint plan names -- see the trailing
NEEDS-MORE-FIXTURES note. Each of the 11 is for real, not padding to hit
a number.
"""
from pathlib import Path

import pytest
from harness.loader import run_default_checks

FIXTURES = Path(__file__).resolve().parents[3] / "spec" / "tests" / "fixtures"

BROKEN_CASES = {
    "unknown_bus.yaml": "E_UNKNOWN_BUS",
    "unknown_actuator.yaml": "E_UNKNOWN_ACTUATOR",
    "duplicate_can_id.yaml": "E_DUP_CAN_ID",
    "peak_below_continuous.yaml": "E_PEAK_BELOW_CONTINUOUS",
    "schema_version_wrong.yaml": "E_SCHEMA_VERSION",
    "unknown_joint_in_pose.yaml": "E_UNKNOWN_JOINT_IN_POSE",
    "hold_torque_exceeded.yaml": "E_HOLD_TORQUE_EXCEEDED",
    "bus_overload.yaml": "E_BUS_OVERLOAD",
    "supply_budget_exceeded.yaml": "E_SUPPLY_BUDGET_EXCEEDED",
    "regen_unset.yaml": "W_REGEN_LIMIT_UNSET",
    "unverified_rating.yaml": "I_UNVERIFIED_RATING",
}


def _run_checks(spec_path: Path) -> list[dict]:
    return [f.model_dump() for f in run_default_checks(spec_path)]


@pytest.mark.parametrize("fixture_name,expected_code", BROKEN_CASES.items())
def test_broken_config_is_flagged_with_right_code(fixture_name: str, expected_code: str):
    path = FIXTURES / "broken" / fixture_name
    findings = _run_checks(path)
    assert findings, f"{fixture_name}: expected at least one finding, got none"
    codes = [f["code"] for f in findings]
    assert expected_code in codes, f"{fixture_name}: expected {expected_code!r}, got {codes!r}"


def test_all_broken_fixtures_are_covered_by_the_parametrized_test():
    broken_dir = FIXTURES / "broken"
    on_disk = {p.name for p in broken_dir.glob("*.yaml")}
    assert on_disk == set(BROKEN_CASES.keys()), (
        "a fixture file exists that isn't in BROKEN_CASES (or vice versa) -- "
        "keep this test's table in sync with the fixtures directory"
    )


def test_known_good_config_has_zero_findings():
    path = FIXTURES / "good" / "robot.yaml"
    findings = _run_checks(path)
    assert findings == []


# --- gates that need real hardware, not static inspection ---


def test_NEEDS_HUMAN_hold_torque_matches_bench_measurement():
    """NEEDS-HUMAN: at the standing pose (or whichever named pose is
    under test), measure actual motor current on the robot and multiply
    by the actuator's torque constant (Kt) to get measured torque.
    Compare against this check's computed required torque for every
    loaded joint; must agree within +-10%. Needs the real robot --
    cannot be run from a config file alone.
    """


def test_NEEDS_HUMAN_bus_load_matches_canbusload_measurement():
    """NEEDS-HUMAN: on the real bus (or a faithful replay), run
    `canbusload` (or equivalent) while the robot is running its actual
    control loop, and compare against `check_bus_load`'s estimate for
    that bus. Must agree within +-5 percentage points. This also
    requires knowing the REAL update rate used on hardware, not just
    check_bus_load's 1000 Hz default -- pass the real rate explicitly
    when running this comparison.
    """


def test_NEEDS_MORE_FIXTURES_to_reach_the_sprint_plans_20():
    """NEEDS-MORE-FIXTURES: the sprint plan names "20 deliberately broken
    configs" as the M2 gate; this suite currently has 11 (one per
    implemented error/warning/info code -- see BROKEN_CASES above).
    Reaching 20 means either adding more distinct violations per
    existing code (e.g. a second way to trigger E_SCHEMA_VERSION) or
    implementing more checks (check_profile_availability is written and
    unit-tested in checks.py but not yet wired into the CLI's default
    check set -- see cli.py's _run_all_checks docstring for why). Not
    done here rather than padded with near-duplicate fixtures just to
    hit a number.
    """
