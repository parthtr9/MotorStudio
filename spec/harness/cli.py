"""`harness` CLI. M0: only `harness check` stub exists; real checks land in M2."""
import click


@click.group()
def main() -> None:
    pass


@main.command()
@click.argument("spec_path")
@click.option("--json", "json_output", is_flag=True, help="JSON output instead of human-readable.")
def check(spec_path: str, json_output: bool) -> None:
    """Run static checks against a robot_hw.yaml. Checks land in M2."""
    raise SystemExit("harness check: not implemented yet (lands in M2)")


if __name__ == "__main__":
    main()
