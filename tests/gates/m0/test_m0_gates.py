"""M0 gate tests.

Per root CLAUDE.md, gate tests are the source of truth for "done" and are
protected (agents must not edit this file). The two M0 gates from the sprint
plan are:

  1. CI on a trivial PR is green in under 5 minutes.
  2. `make vcan-up && make test` passes with no manual steps on a clean
     Ubuntu 24.04 machine.

Gate 1 can only be proven by actually opening a PR and watching CI, and
gate 2's vcan half is Linux-only (SocketCAN) so it can't run on this dev
machine (macOS). Both are marked NEEDS-HUMAN / NEEDS-CI below; this file
verifies the parts that *can* be checked by static inspection so a
regression (missing workflow, renamed Makefile target, etc.) is caught by
`make gates M=0`.
"""
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[3]


def test_ci_workflow_exists_and_has_required_jobs():
    wf_path = ROOT / ".github" / "workflows" / "ci.yml"
    assert wf_path.exists(), "CI workflow missing: .github/workflows/ci.yml"
    wf = yaml.safe_load(wf_path.read_text())
    assert "vcan-rig" in wf["jobs"], "ci.yml must run `make vcan-up && make test` in a job"
    assert "spec" in wf["jobs"]


def test_makefile_has_required_targets():
    mk = (ROOT / "Makefile").read_text()
    for target in ("vcan-up:", "test:", "gates:"):
        assert target in mk, f"Makefile missing target {target!r}"


def test_vcan_up_script_exists_and_is_executable():
    script = ROOT / "scripts" / "vcan-up.sh"
    assert script.exists(), "scripts/vcan-up.sh missing"
    assert script.stat().st_mode & 0o111, "scripts/vcan-up.sh must be executable"


def test_spec_schema_v0_importable():
    import sys

    sys.path.insert(0, str(ROOT / "spec"))
    from harness.schema import RobotSpec

    assert RobotSpec.model_fields["schema_version"].default == 0


def test_agent_files_committed():
    for rel in (
        "CLAUDE.md",
        "driver/CLAUDE.md",
        "spec/CLAUDE.md",
        ".claude/agents/safety-reviewer.md",
        ".claude/agents/profile-author.md",
        ".claude/agents/gate-runner.md",
        ".claude/commands/milestone.md",
        ".claude/commands/add-motor.md",
        ".claude/commands/check-spec.md",
        ".claude/settings.json",
        ".claude/hooks/guard.py",
    ):
        assert (ROOT / rel).exists(), f"agent file missing: {rel}"


def test_license_and_contributing_present():
    assert (ROOT / "LICENSE").exists()
    assert "Apache License" in (ROOT / "LICENSE").read_text()
    assert (ROOT / "CONTRIBUTING.md").exists()


# --- gates that need a human or a CI run, not static inspection ---

def test_NEEDS_HUMAN_ci_trivial_pr_under_5_minutes():
    """NEEDS-HUMAN: open a trivial PR against this repo's default branch on
    GitHub and confirm the Actions run finishes green in under 5 minutes.
    Command: `git checkout -b m0-trivial && git commit --allow-empty -m
    "trivial" && git push -u origin m0-trivial` then open the PR and watch
    the Actions tab.
    """


def test_NEEDS_HUMAN_vcan_up_on_clean_ubuntu_24_04():
    """NEEDS-HUMAN: on a clean Ubuntu 24.04 machine/container with no prior
    setup, run `make vcan-up && make test` and confirm it passes with no
    manual steps (sudo password prompts aside). This dev machine is macOS,
    where vcan (SocketCAN) is unavailable, so `scripts/vcan-up.sh` exits 1
    by design here -- that is expected, not a failure of this gate.
    """
