# Damiao intake -- nothing here is sourced yet

Per root `CLAUDE.md`: "Unknown frame or param -> log + reject. Never guess."
No `profiles/damiao/*.yaml` exists yet because no Damiao CAN protocol detail
has been confirmed against a manual or a real capture in this repo. This
file tracks what's needed before `profile-author` can draft one -- same
intake this project already ran for RobStride, see
`profiles/robstride/OPEN_QUESTIONS.md`.

Models in scope: UNCONFIRMED. `motor_vendor_damiao` (the motorbridge crate
driver/src/backends/damiao.rs wraps, see its `DAMIAO_MODELS` table in
motor.rs) currently recognizes 3507, 4310, 4310P, 4340, 4340P, 4340_v20,
6006, 8006, 8009, 10010L, 10010, H3510, G6215, H6220, JH11, 6248P -- that is
motorbridge's supported list, not a statement of which of these this
project actually owns or plans to run. Confirm the real in-scope subset
with a human before drafting any profile.

## Needed per model, before a profile can be drafted

- [ ] Damiao <model> -- manual (CAN protocol section specifically, not just
      the mechanical/electrical datasheet)
- [ ] Damiao <model> -- firmware version actually flashed on the unit(s) we
      have (protocol fields can change per firmware -- see the plan doc's
      research section for the CubeMars precedent; unknown for Damiao until
      we see version history for our own units)
- [ ] Damiao <model> -- a real `candump -L` capture of at least a few
      minutes covering: scan/discovery (note: motor_vendor_damiao has no
      ping/discovery method at all per driver/src/backends/damiao.rs's
      module doc -- confirm what "discovery" even means on real hardware
      before assuming the capture needs to show it), enable, a jog through
      MIT/PosVel/Vel/ForcePos modes if supported, a disable, and at least
      one fault/error condition if triggerable safely -> saved to
      `captures/damiao/<model>_<fw>.log` (captures/ is read-only once
      added; never hand-edited)

## What happens once a manual lands

`profile-author` subagent (`.claude/agents/profile-author.md`) drafts
`profiles/damiao/<model>_<fw>.yaml` against `ProtocolProfile`
(`spec/harness/profile_schema.py`). Every frame and every field must carry
a `manual_ref` citing the specific section/page/table. Anything the manual
doesn't make unambiguous goes in that profile's `open_questions:` list
instead of being guessed -- `harness profile-lint <path>` reports the
count. The profile is then checked against the real capture via `make
replay` (M1 gate: 100% of frames decoded, 0 unknown, round-trip within 1
LSB) before it's considered usable.

## Note on motorbridge

Per the sprint plan's feature evaluation, Damiao is a "Reuse" target --
`motorbridge` already has a Damiao backend, and M1's driver layer wraps it
(`driver/src/backends/damiao.rs`) as a `Backend` rather than writing a new
wire codec. The `profiles/damiao/*.yaml` described above is still wanted as
a documented, citeable lookup (for limits/checks/Studio display and so a
human can audit what the system believes about the protocol) independent
of whether the bytes are actually encoded by motorbridge or by a
from-scratch codec. If motorbridge's own source turns out to be the only
available reference for some field (no vendor manual section covers it),
that must be recorded explicitly as the source in `manual_ref` -- e.g.
`"motorbridge motor_vendors/damiao/src/protocol.rs L114, no vendor manual
section found"` -- never presented as if a manual confirmed it. (This
already applies to the status-code table damiao.rs cites in its own module
doc: that's read from motorbridge's `protocol.rs::status_name`, not a
vendor manual page.)
