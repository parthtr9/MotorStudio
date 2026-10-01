# actuator-harness

Spec-driven actuator toolkit. One spec file -> static checks, runtime gateway, Studio GUI, MCP server.
Protects EQUIPMENT. Not a human safety system. Never claim otherwise in code, docs, or commits.

## Layout
- driver/    Rust. backends (motorbridge, yaml-codec), transports, gateway daemon
- spec/      Python. spec loader, URDF import, limit compiler, checks, CLI `harness`
- studio/    TypeScript. web UI served by gateway over WebSocket
- mcp/       Python. MCP server, talks to gateway only
- profiles/  YAML protocol profiles, one per family+model+firmware
- captures/  real candump logs. read-only. never edit, never regenerate
- tests/gates/  milestone gate tests. source of truth for "done"

## Commands
- make vcan-up        bring up vcan0/vcan1
- make test           all unit tests, no hardware
- make gates M=2      gate tests for milestone M (sim/replay only)
- make check SPEC=x   run spec checks
- cargo test -p driver ; pytest spec mcp ; pnpm -C studio test

## Hard rules
- NO real hardware. Only vcan*, replay transport, emulator. Hardware = human runs it.
- Only the gateway writes to a bus. No other code opens a CAN socket for writing.
- Do not edit: driver/src/gateway/enforce/, captures/, tests/gates/. Propose changes in PR text instead.
- Never loosen a limit, default, or tolerance to make a test pass. Fix the code or report.
- New limit or check -> test for violating AND passing case, same PR.
- Units SI everywhere (rad, rad/s, Nm, A, V, degC, s). Convert at the profile boundary only.
- Profiles are data. No family-specific branches outside driver/src/backends/.
- Unknown frame or param -> log + reject. Never guess.

## Workflow
1. Read the milestone gate tests first.
2. Plan in PR description: files, tests, risks.
3. Small PRs. One feature each.
4. Done = make test + make gates M=<current> green. Paste summary.

## Output style
Terse. Bullets. What changed, what tested, what failed, what next. No filler.
