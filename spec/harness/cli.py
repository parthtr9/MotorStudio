"""`harness` CLI."""
import json as json_module
import sys

import click

from harness.checks import (
    check_bus_load,
    check_hold_torque,
    check_supply_regen_budget,
    check_unverified_ratings,
    findings_from_validation_error,
)
from harness.limits_compiler import compile_effective_limits
from harness.loader import SpecLoadError, load_robot
from harness.profile_loader import load_profile


@click.group()
def main() -> None:
    pass


def _run_all_checks(spec_path: str) -> list[dict]:
    """Loads the spec and runs every default check, returning
    JSON-serializable Finding dicts. Schema/load errors short-circuit
    the semantic checks -- they can't run meaningfully against a spec
    that didn't even parse.

    check_profile_availability is NOT included here yet: no real
    per-family profile exists on disk for any actuator this project
    currently targets (see profiles/robstride/OPEN_QUESTIONS.md), so
    wiring it in now would make every config -- including a
    well-formed one -- emit a profile-missing warning. Revisit once
    real profiles land.
    """
    try:
        spec, urdf = load_robot(spec_path)
    except SpecLoadError as e:
        return [f.model_dump(mode="json") for f in findings_from_validation_error(e.validation_error)]

    findings = []
    findings += check_hold_torque(spec, urdf)
    findings += check_bus_load(spec)
    findings += check_supply_regen_budget(spec)
    findings += check_unverified_ratings(spec)
    return [f.model_dump(mode="json") for f in findings]


@main.command()
@click.argument("spec_path")
@click.option("--json", "json_output", is_flag=True, help="JSON output instead of human-readable.")
def check(spec_path: str, json_output: bool) -> None:
    """Run static checks against a robot_hw.yaml."""
    findings = _run_all_checks(spec_path)
    if json_output:
        click.echo(json_module.dumps(findings, indent=2))
    else:
        if not findings:
            click.echo("0 findings.")
        for f in findings:
            joint_part = f" [{f['joint']}]" if f.get("joint") else ""
            click.echo(f"{f['severity'].upper():7} {f['code']:28}{joint_part}  {f['message']}")
            click.echo(f"        fix: {f['fix']}")
    if any(f["severity"] == "error" for f in findings):
        sys.exit(1)


@main.command()
@click.argument("spec_path")
@click.option("-o", "--output", "output_path", default="limits.json", help="Output path.")
def compile(spec_path: str, output_path: str) -> None:
    """Compile a robot_hw.yaml + its URDF into limits.json for the gateway."""
    try:
        spec, urdf = load_robot(spec_path)
    except SpecLoadError as e:
        for f in findings_from_validation_error(e.validation_error):
            click.echo(f"ERROR {f.code}: {f.message}", err=True)
        sys.exit(1)
    limits = compile_effective_limits(spec, urdf)
    with open(output_path, "w") as out:
        json_module.dump(limits, out, indent=2)
    click.echo(f"wrote {output_path}")


@main.command("profile-lint")
@click.argument("profile_path")
def profile_lint(profile_path: str) -> None:
    """Validate a profiles/<family>/<model>_<fw>.yaml and print coverage.

    Every frame/field must already cite a manual_ref or this fails to load
    (pydantic ValidationError) -- that is the check. This command then
    reports how much of the family is documented vs still open.
    """
    profile = load_profile(profile_path)
    summary = profile.coverage_summary()
    click.echo(f"{summary['family']}/{summary['model']}_{summary['firmware']}")
    click.echo(f"  frames documented: {summary['frames_documented']}")
    click.echo(f"  params documented: {summary['params_documented']}")
    click.echo(f"  open questions:    {summary['open_questions']}")
    if profile.open_questions:
        click.echo("  --")
        for q in profile.open_questions:
            click.echo(f"  ? {q}")


if __name__ == "__main__":
    main()
