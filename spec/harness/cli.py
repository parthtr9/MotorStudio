"""`harness` CLI. M0: `harness check` stub. M1: `harness profile-lint`."""
import click

from harness.profile_loader import load_profile


@click.group()
def main() -> None:
    pass


@main.command()
@click.argument("spec_path")
@click.option("--json", "json_output", is_flag=True, help="JSON output instead of human-readable.")
def check(spec_path: str, json_output: bool) -> None:
    """Run static checks against a robot_hw.yaml. Checks land in M2."""
    raise SystemExit("harness check: not implemented yet (lands in M2)")


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
