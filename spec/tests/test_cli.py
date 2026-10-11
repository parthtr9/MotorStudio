"""CLI-layer tests for `harness`. harness/checks.py, limits_compiler.py and
profile_schema.py each have their own unit tests for the underlying pure
logic -- these exercise the click commands themselves (argument parsing,
file I/O wiring, output formatting, exit codes), which had zero coverage.
"""
from pathlib import Path

import yaml
from click.testing import CliRunner
from harness.cli import main

FIXTURES = Path(__file__).resolve().parent / "fixtures"
GOOD_SPEC = FIXTURES / "good" / "robot.yaml"
BROKEN_SPEC = FIXTURES / "broken" / "bus_overload.yaml"


def _minimal_profile_dict() -> dict:
    return {
        "family": "testfam",
        "model": "X1",
        "firmware": "0.0",
        "manual_title": "Testfam Fake Manual for unit tests",
        "frames": [
            {
                "name": "status_feedback",
                "direction": "rx",
                "can_id": 0x100,
                "extended": False,
                "dlc": 2,
                "fields": [
                    {
                        "name": "position",
                        "byte_offset": 0,
                        "byte_length": 2,
                        "byte_order": "big",
                        "scale": 0.01,
                        "unit": "rad",
                        "manual_ref": "Fake Manual sec 1.1",
                    }
                ],
                "manual_ref": "Fake Manual sec 1.1",
            }
        ],
        "open_questions": ["Does firmware 0.0 differ from 0.1? Unconfirmed."],
    }


# --- check ---


def test_check_on_good_spec_reports_zero_findings_and_exits_0():
    result = CliRunner().invoke(main, ["check", str(GOOD_SPEC)])
    assert result.exit_code == 0
    assert "0 findings." in result.output


def test_check_on_broken_spec_prints_finding_and_exits_1():
    result = CliRunner().invoke(main, ["check", str(BROKEN_SPEC)])
    assert result.exit_code == 1
    assert "E_BUS_OVERLOAD" in result.output


def test_check_json_output_is_valid_json_matching_human_output_findings():
    result = CliRunner().invoke(main, ["check", str(BROKEN_SPEC), "--json"])
    assert result.exit_code == 1
    import json

    findings = json.loads(result.output)
    assert any(f["code"] == "E_BUS_OVERLOAD" for f in findings)


# --- compile ---


def test_compile_on_good_spec_writes_limits_json():
    runner = CliRunner()
    with runner.isolated_filesystem():
        result = runner.invoke(main, ["compile", str(GOOD_SPEC), "-o", "limits.json"])
        assert result.exit_code == 0
        assert Path("limits.json").exists()
        import json

        limits = json.loads(Path("limits.json").read_text())
        assert any(j["joint"] == "knee_joint" for j in limits["joints"])


def test_compile_on_unparseable_spec_prints_named_reason_and_exits_1_without_writing():
    runner = CliRunner()
    with runner.isolated_filesystem():
        broken = dict(yaml.safe_load(GOOD_SPEC.read_text()))
        broken["schema_version"] = 999  # wrong version -> E_SCHEMA_VERSION
        Path("bad.yaml").write_text(yaml.dump(broken))
        result = runner.invoke(main, ["compile", "bad.yaml", "-o", "limits.json"])
        assert result.exit_code == 1
        assert "E_SCHEMA_VERSION" in result.output
        assert not Path("limits.json").exists()


# --- profile-lint ---


def test_profile_lint_on_valid_profile_prints_coverage_summary():
    runner = CliRunner()
    with runner.isolated_filesystem():
        Path("profile.yaml").write_text(yaml.dump(_minimal_profile_dict()))
        result = runner.invoke(main, ["profile-lint", "profile.yaml"])
        assert result.exit_code == 0
        assert "testfam/X1_0.0" in result.output
        assert "frames documented: 1" in result.output
        assert "open questions:    1" in result.output
        assert "Does firmware 0.0 differ" in result.output


def test_profile_lint_on_invalid_profile_prints_named_reason_not_a_traceback():
    # Dropping manual_ref from a field must fail to load -- that IS the
    # check (per profile_schema.py's doc) -- and must surface as a clean
    # message, not a raw pydantic traceback dumped to the terminal.
    runner = CliRunner()
    with runner.isolated_filesystem():
        profile = _minimal_profile_dict()
        del profile["frames"][0]["fields"][0]["manual_ref"]
        Path("profile.yaml").write_text(yaml.dump(profile))
        result = runner.invoke(main, ["profile-lint", "profile.yaml"])
        assert result.exit_code == 1
        assert result.exception is None or isinstance(result.exception, SystemExit)
        assert "manual_ref" in result.output
        assert "Traceback" not in result.output
