# RobStride intake -- nothing here is sourced yet

Per root `CLAUDE.md`: "Unknown frame or param -> log + reject. Never guess."
No `profiles/robstride/*.yaml` exists yet because no RobStride CAN protocol
detail has been confirmed against a manual or a real capture in this repo.
This file tracks what's needed before `profile-author` can draft one.

Models in scope (per user): RobStride 01, 02, 03, 04, 05, 06.

## Needed per model, before a profile can be drafted

- [ ] RobStride 0X -- manual (CAN protocol section specifically, not just
      the mechanical/electrical datasheet)
- [ ] RobStride 0X -- firmware version actually flashed on the unit(s) we
      have (protocol fields can change per firmware, per the plan doc's
      research section -- CubeMars is the documented example, RobStride may
      or may not have the same issue; unknown until we see version history)
- [ ] RobStride 0X -- a real `candump -L` capture of at least a few minutes
      covering: scan/discovery, enable, a jog through position/velocity/
      torque modes if supported, a disable, and at least one fault/error
      condition if triggerable safely -> saved to
      `captures/robstride/<model>_<fw>.log` (captures/ is read-only once
      added; never hand-edited)

## What happens once a manual lands

`profile-author` subagent (`.claude/agents/profile-author.md`) drafts
`profiles/robstride/<model>_<fw>.yaml` against `ProtocolProfile`
(`spec/harness/profile_schema.py`). Every frame and every field must carry
a `manual_ref` citing the specific section/page/table. Anything the manual
doesn't make unambiguous goes in that profile's `open_questions:` list
instead of being guessed -- `harness profile-lint <path>` reports the
count. The profile is then checked against the real capture via `make
replay` (M1 gate: 100% of frames decoded, 0 unknown, round-trip within 1
LSB) before it's considered usable.

## Note on motorbridge

Per the sprint plan's feature evaluation, RobStride is a "Reuse" target --
`motorbridge` already has a RobStride backend, and M1's driver layer wraps
it as a `Backend` rather than writing a new wire codec. The
`profiles/robstride/*.yaml` described above is still wanted as a documented,
citeable lookup (for limits/checks/Studio display and so a human can audit
what the system believes about the protocol) independent of whether the
bytes are actually encoded by motorbridge or by a from-scratch codec. If
motorbridge's own source turns out to be the only available reference for
some field (no vendor manual section covers it), that must be recorded
explicitly as the source in `manual_ref` -- e.g. `"motorbridge
src/backends/robstride.rs L42, no vendor manual section found"` -- never
presented as if a manual confirmed it.
