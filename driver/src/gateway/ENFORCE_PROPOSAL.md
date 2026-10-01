# `enforce/` design proposal (for human review -- not committed code)

Per root `CLAUDE.md`: `driver/src/gateway/enforce/` is human-owned,
hook-protected (`.claude/hooks/guard.py` blocks Write/Edit there), and
must be "reviewed line by line" (sprint plan risk table: "An agent-written
enforcement bug burns a motor"). This file is a proposal for what goes in
there, written against the types already defined in `gateway/types.rs`
and `gateway/arm.rs` -- which are safe for an agent to write, since they
carry no clamp/trip decision themselves. Nothing below is committed logic.
Treat it as a starting point for whoever authors `enforce/`, to accept,
change, or discard.

## Scope

Per `driver/CLAUDE.md`: "Every command: decode -> enforce -> encode ->
send." `enforce/` is the middle step -- it implements the `Enforcer`
trait from `gateway/types.rs`:

```rust
pub trait Enforcer {
    fn enforce(
        &mut self,
        joint: &str,
        requested: Command,
        limits: &JointLimits,
    ) -> EnforceDecision;
}
```

One call per joint per command tick. Must be `enforce/`'s own
implementation, not a wrapper that calls out to anything outside this
module -- the whole point is one reviewable place.

## 1. Position/velocity/torque clamps

Straightforward saturation against `JointLimits`:

```text
clamped.position_rad = requested.position_rad.map(|p| p.clamp(limits.min_position_rad, limits.max_position_rad))
clamped.velocity_rad_s = requested.velocity_rad_s.map(|v| v.clamp(-limits.max_velocity_rad_s, limits.max_velocity_rad_s))
clamped.torque_nm = requested.torque_nm.map(|t| t.clamp(-limits.max_torque_nm, limits.max_torque_nm))
```

If any field was actually changed by `.clamp()`, emit a `TripLogEntry`
with `action` = `ClampedPosition`/`ClampedVelocity`/`ClampedTorque` (one
entry per field that changed, not just the first) and `limit` naming
which `JointLimits` field triggered it (e.g. `"max_torque_nm"`).

**Open question for the human author**: when a command sets multiple
fields and more than one clamps, should that produce multiple
`TripLogEntry`s in one call, or one entry describing all of them? The
type as written (`EnforceDecision.trip: Option<TripLogEntry>`, singular)
assumes at most one entry per `enforce()` call -- if multiple simultaneous
clamps need to be loggable individually, that type needs to change to
`Vec<TripLogEntry>` first. Don't work around the type; fix it if the
single-`Option` shape is wrong.

## 2. Per-motor I²t

Per the sprint plan: "Industry-standard thermal guard needing no thermal
constants" (unlike the duty-profile thermal prediction feature, which is
explicitly v1.1 and marked unverified pending bench data). Classic motor
I²t accumulator, run once per joint per tick:

```text
i2t_state.accumulator += ((measured_current_a / limits.continuous_current_a)^2 - 1.0) * dt_s
i2t_state.accumulator = i2t_state.accumulator.max(0.0)  // don't let headroom "bank" during idle
```

Two thresholds to decide (not invented here -- needs a real value, from
the family's datasheet/profile or a bench-validated constant, same "never
guess" rule as protocol data):

- **Derate threshold**: above this accumulator value, scale down the
  allowed torque/current ceiling before tripping outright. Action =
  `I2tDerated`.
- **Trip threshold**: above this, refuse the commanded torque/current
  entirely (clamp toward zero) until the accumulator decays back down.
  Action = `I2tTripped`.

**Open question**: where does `measured_current_a` come from?
`Feedback` (in `backend.rs`) carries `torque_nm`, not current directly --
converting needs the motor's torque constant (`Nm/A`), which lives in the
spec's `ActuatorRating`, not in `JointLimits` as currently defined.
Either add a `torque_constant_nm_per_a` field to `JointLimits` (so the
effective-limit compiler, M2, bakes it in) or add a current field to
`Feedback`. Pick one deliberately -- don't let `enforce/` reach past its
own inputs to get it.

The accumulator itself is per-joint runtime state that outlives any
single `enforce()` call, so `Enforcer`'s concrete struct needs a
`HashMap<String, I2tState>` (keyed by joint) or similar -- the trait
signature (`&mut self`) already allows for this.

## 3. Bus current budget

Sum of commanded currents across all joints sharing a bus must not exceed
`BusLimits.supply_current_limit_a`. This needs cross-joint state (the
`Enforcer` sees one joint per call), so either:

- `enforce()` is called once per joint per tick but the budget check runs
  in a separate pass after all joints' calls for that tick, with the
  per-joint results fed back in, or
- the trait gets a second method for a whole-bus pass, e.g.
  `fn enforce_bus_budget(&mut self, bus: &str, joint_commands: &[(String, Command)], bus_limits: &BusLimits) -> Vec<EnforceDecision>`.

**This is a real gap in the current `Enforcer` trait shape** -- it was
designed assuming one joint is independent of others, which bus budget
enforcement violates. Resolve this before implementing, not during.

Regen budget (`BusLimits.regen_current_limit_a`) is the same shape of
problem, but only applies when commanded torque opposes measured
velocity (braking) -- sign-check before summing into the regen side vs.
the drive side.

## 4. Command rate limit

Track last-accepted-command timestamp per joint; if the new command
arrives faster than `1.0 / EffectiveLimits.command_rate_limit_hz` after
the last one, either drop it (and hold the previous command) or clamp the
*rate of change* of the commanded value rather than the value itself --
these have very different behavior for a leg mid-stride and the sprint
plan doesn't specify which. Flag to the human: decide this, don't let an
agent pick silently.

## 5. Watchdog + fault policy

Separate concern from per-command enforcement: if no new command arrives
for a joint within some timeout, apply `JointLimits.fault_policy`:

- `Damp`: ramp torque toward zero over some decay time (not instant zero
  -- per the sprint plan, "Leg robots need a slow sag, not a drop").
- `ZeroTorque`: immediate zero torque command.
- `Hold`: freeze at last commanded position (position-mode hold).

This needs a clock/timer independent of `enforce()` being called (if
commands stop arriving, nothing calls `enforce()` to notice) -- likely a
separate periodic task in the gateway daemon (M3) that checks
last-command-time per joint and, on timeout, synthesizes a fault command
and routes it through the normal `encode -> send` path. `TripLogEntry`
action = `WatchdogFault(policy)`.

**Per-family behavior differs and must be tested each time** (sprint
plan risk table) -- the watchdog deciding to send a "ramp to zero"
command is itself a command that still has to go through a real
`Backend`, and whether that backend can actually execute a smooth ramp
(vs. only accepting single-shot commands) is family-specific and
unverified until there's a real backend to test against.

## What this proposal deliberately does not decide

- Exact I²t derate/trip thresholds (needs real data, not a guess).
- Whether multiple simultaneous clamps produce one or many trip-log
  entries (needs a type change, flagged above).
- Bus-budget's cross-joint aggregation shape (needs a trait change,
  flagged above).
- Command-rate-limit behavior: drop vs. rate-limit-the-delta.
- Watchdog decay time constants for `Damp`.

Each of these is a real decision with safety consequences, which is
exactly why this stays a proposal and not code.
