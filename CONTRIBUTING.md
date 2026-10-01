# Contributing

This protects EQUIPMENT. It is not a human safety system. Do not present it as one in code, docs, commits, or PRs.

## Setup
```
make vcan-up
make test
```
No real hardware required or wanted for any contribution. See `NO real hardware` in root `CLAUDE.md`.

## Workflow
1. Read the relevant milestone's gate tests in `tests/gates/m<N>/` before starting.
2. Open a PR description with: files touched, tests added, risks.
3. Keep PRs small, one feature each.
4. `make test` and `make gates M=<current>` must be green before merge.
5. Any PR touching `driver/src/gateway/`, `spec/harness/checks*`, or `profiles/` needs a safety review (see `.claude/agents/safety-reviewer.md`) before merge.

## Adding a motor family or profile
Use `.claude/commands/add-motor.md`. A profile with no matching real capture under `captures/` will not be merged.

## Hard rules (see root CLAUDE.md for full list)
- Only the gateway writes to a CAN bus.
- Never loosen a limit, default, or tolerance to make a test pass.
- New limit or check needs a violating-case test and a passing-case test, same PR.
- SI units everywhere; vendor units only inside profiles/codec.

## Code of conduct
Be direct, be kind, assume good faith. Report issues to the maintainer.
