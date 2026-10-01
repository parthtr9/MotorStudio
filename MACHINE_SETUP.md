# Machine setup

This file is **only** for bringing this repo up on a machine -- toolchain
install, per-OS quirks, and the one-time fixup a fresh clone needs. It is
not contributor workflow (see `CONTRIBUTING.md`) and not agent behavior
rules (see `CLAUDE.md`). If you're adopting this harness on a new laptop,
start here.

## Why this file exists

The user runs this harness on multiple laptops. Machine-specific setup
information (exact paths, toolchain versions, OS differences) doesn't
belong in `CLAUDE.md` (agent rules, should stay machine-agnostic) or
`CONTRIBUTING.md` (contributor workflow, assumes setup already works). It
lives here so a new machine can be brought up without re-deriving any of
this from scratch or hitting the same bugs again.

## Required on every machine

| Tool | Why | Version last verified against |
| --- | --- | --- |
| Python 3.11+ | `spec/` (pydantic schema, checks, CLI, MCP server) | 3.14.7 |
| Rust (`cargo`, `rustc`) | `driver/` (gateway daemon, backends) | 1.98.1 |
| Node.js + `pnpm` | `studio/` (Studio GUI, once it exists) | Node v24.9.0, pnpm 8.15.2 |
| `git` | obviously | -- |
| `make` | all top-level commands go through the `Makefile` | -- |

### macOS

```bash
# Homebrew, if not already installed:
#   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install rust          # gives you cargo + rustc
# Python 3.11+ and node/pnpm are commonly already present via brew/nvm;
# if not: brew install python node && npm install -g pnpm
```

**vcan (SocketCAN) does NOT work on macOS.** It's Linux-only kernel
functionality. `make vcan-up` will fail loudly and on purpose on macOS
(`scripts/vcan-up.sh` checks `uname -s` and exits 1 with an explanation
rather than doing something wrong silently). On macOS, use the replay
transport for anything that needs a simulated bus. `make test` and
`make gates M=<n>` do not require vcan.

### Linux (e.g. Ubuntu 24.04, the CI target)

```bash
sudo apt update
sudo apt install -y build-essential python3 python3-venv python3-pip make git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # installs cargo/rustc
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs
npm install -g pnpm
```

vcan works here:

```bash
make vcan-up   # modprobes vcan, brings up vcan0 + vcan1
```

This is also what M0's gate test requires: "`make vcan-up && make test`
passes with no manual steps on a clean Ubuntu 24.04 machine." If you're
verifying that gate, do it on an actual clean Ubuntu 24.04 box/container,
not a machine that already has state from other projects.

## After cloning to a new machine or a new path

**Run this once, every time the repo lands at a new absolute path**
(a different machine, a different user account, a different directory on
the same machine):

```bash
./scripts/fix-hook-paths.sh
```

### Why this is necessary -- read this before skipping it

`.claude/settings.json` configures two PreToolUse hooks (`guard.py`,
which blocks real-hardware CAN commands and edits to protected paths) and
one PostToolUse hook (`make quick-test`). **These hook commands use
absolute paths, not relative ones, on purpose.**

This repo hit a real lockout bug during initial setup: the hooks
originally used relative paths (`python3 .claude/hooks/guard.py`). A
`cd driver && cargo init` left the session's shell working directory
inside `driver/`. Every subsequent Bash/Edit/Write/MultiEdit call then
failed before it even ran, because Claude Code couldn't find
`driver/.claude/hooks/guard.py` (it doesn't exist -- the real one is at
the repo root). The agent was fully locked out of all three of those
tools until a human (or a separately-launched agent whose working
directory happened to start somewhere else) fixed `settings.json`
directly.

Switching to absolute paths fixes that specific failure mode -- the hook
resolves correctly no matter what the shell's cwd drifts to mid-session.
But it introduces a different, narrower problem: the absolute path is
specific to the machine and directory it was written on. Clone the repo
to a new machine, or move it to a different path on the same machine, and
the hooks silently point at a path that doesn't exist there -- same
lockout, different cause. `scripts/fix-hook-paths.sh` rewrites
`.claude/settings.json` to use wherever the repo actually is on disk
(`$(cd "$(dirname "$0")/.." && pwd)`), so it's always correct to re-run
after a move. It's idempotent -- safe to run even if the paths are
already correct.

If you ever see tool calls failing with something like:

```
PreToolUse:Bash hook error: [python3 .claude/hooks/guard.py]: ... No such file or directory
```

that's this exact failure mode. Run `scripts/fix-hook-paths.sh` (from a
session that can still run Bash/Edit -- if literally every tool is
locked, you'll need to edit `.claude/settings.json` by hand, or just run
`cd <repo-root>` first since the relative-path version of the bug only
bites after cwd drifts away from the repo root).

## Verifying a new machine is set up correctly

```bash
make test          # spec/ unit tests; driver/studio tests run too once
                    # Cargo.toml / package.json exist for them
make gates M=0      # M0 foundation gates
```

Both should pass clean with no manual intervention beyond the toolchain
install above.

## Vendor manuals (not in git)

`Robstride/` (the RobStride RS00-RS06 + RS02-IP67 user manual PDFs) is
gitignored -- they're copyrighted vendor documents and this project plans
a public release (see the sprint plan's M6), so they must never enter git
history. Each machine that needs to re-derive or re-check protocol
extracts against the source PDFs needs its own copy of
`Robstride/*.pdf`, placed at the repo root, copied over out-of-band (not
via git). The derived, page-cited extracts that **are** version-controlled
live in `profiles/robstride/_extracts/*.md` and don't require the PDFs to
be present.

## Known OS/environment gotchas

- **Apple Silicon vs Intel Mac**: Homebrew lives at `/opt/homebrew` on
  Apple Silicon, `/usr/local` on Intel. If a script or doc ever hardcodes
  a brew path, check which one applies.
- **vcan is Linux-only.** Don't try to make it work on macOS; use the
  replay transport (`driver/`'s replay transport, once built in M1)
  instead for anything that needs a simulated bus on a Mac.
- **Edition 2024 Rust**: `driver/Cargo.toml` was generated with
  `edition = "2024"` by a recent `cargo init`. If a machine's Rust
  toolchain predates edition 2024 support, either update Rust or change
  the edition field -- don't silently downgrade correctness/lints to
  dodge this, per `CLAUDE.md`'s "fix the code or report" rule.
