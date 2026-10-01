# Actuator Harness: Research, Feature Evaluation, and Sprint Plan

Sep 30, 2026 · @Parth

## Summary

Build the spec-and-limits layer, not another motor driver. Multi-vendor CAN drivers already exist and are active, so the universal Motor Studio should sit on top of them. Nobody has shipped the part that reads your motor ratings and URDF, predicts thermal trouble, and blocks unsafe commands at runtime.

The product is one spec file plus four things built on it:

1. **Studio:** a GUI that discovers, configures and jogs motors from any adapter.
2. **Checks:** static analysis of the spec before arming, led by the hold-torque check.
3. **Gateway:** the only process that writes to the bus, enforcing clamps, I²t, bus budget and watchdog.
4. **Agent layer:** an MCP server and sim twin so coding agents get structured feedback and hard limits.

The sprint plan runs six milestones over about six weeks of part-time work. Each milestone ends in a test gate you either pass or fail. The agent files in the last sections are ready to copy into the repo.

The research leaned on vendor forums, GitHub issues and builder write-ups. Optimist and pessimist cases for each feature sit in the evaluation table.

## What builders are running into

Seven pain points came up repeatedly. Each one maps to a feature.

**1. Protocol bring-up fails silently.** A CubeMars library reported MIT mode as enabled while the motor produced no torque, because motors reply to MIT frames whether enabled or not ([TUM Aries PR #73](https://github.com/TUM-Aries-Lab/motor-module/pull/73)). Arduino users report motors that read fine but ignore servo-mode commands ([Arduino forum](https://forum.arduino.cc/t/cubemars-ak10-9-works-in-mit-mode-but-not-in-servo-mode-using-arduino-uno-can-bus-shield/1437008), [MyBotShop forum](https://forum.mybotshop.de/t/ak80-64-mit-mode-cannot-control/414)).

**2. Protocol details change with firmware.** One C++ driver keeps separate MIT scaling constants per firmware version of the same AK80-6 motor ([well-robotics](https://github.com/well-robotics/cubemars-tmotor-can)). CubeMars' own FAQ says feedback is request-response in one tool version and configurable in the next ([CubeMars FAQ](https://www.cubemars.com/faq)). Servo mode only answers extended 29-bit frames ([CubeMars Q&A](https://www.cubemars.com/article-262-AK+Series.html)).

**3. Every vendor has its own tool and dongle.** CyberGearDashboard exists because the stock Xiaomi software only works with its own adapter ([CyberGearDashboard](https://github.com/jgillick/CyberGearDashboard)). RobStride and CyberGear alone have more than a dozen separate libraries ([GitHub topic](https://github.com/topics/cybergear)).

**4. Regen and power misconfiguration kills hardware.** ODrive's founder called undumped regen energy one of the few things sure to destroy the FETs ([ODrive #77](https://github.com/madcowswe/ODrive/issues/77)). Users fry drives by leaving regen current limits at the wrong value ([ODrive forum](https://discourse.odriverobotics.com/t/braking-current-and-fried-odrive/7459)), and supplies cut out during braking ([ODrive forum](https://discourse.odriverobotics.com/t/odrive-cuts-out-when-braking/6638)).

**5. Holding a pose overheats leg motors.** Humanoid knees can hit their thermal limit in under 2 minutes while standing ([Firgelli guide](https://www.firgelli.com/pages/humanoid-robot-actuators)). James Bruton's quadruped knees got too hot to touch just staying upright ([Hackaday](https://hackaday.com/tag/robot-actuator/)). CubeMars' own knee guide warns about ignoring thermal stability ([CubeMars](https://www.cubemars.com/how-to-choose-hip-and-knee-joint-motors-for-humanoid-robots.html)).

**6. Bus load and IDs bite at scale.** A K-Bot builder measured six RobStrides at 100 Hz using about 15% of a 1 Mbps bus and planned multiple buses for 20+ motors ([dev.to](https://dev.to/infoslack/building-my-humanoid-robot-pdg)). Damiao guidance says to set IDs one motor at a time to avoid collisions ([Damiao guide](https://www.roboticscenter.ai/wiki/damiao-motors)). Python control loops can starve the receive thread and look like a dead motor ([cubemarsPyCAN](https://github.com/verlab/cubemarsPyCAN)).

**7. Hardware connection is the hard part for agents too.** In Anthropic's robodog study, connecting to the hardware was the step both teams struggled with most ([Project Fetch](https://www.anthropic.com/research/project-fetch-robot-dog)). Agents already iterate well in simulation ([Claude Code in sim](https://medium.com/@brianytsui/claude-code-as-embodied-agent-to-control-robots-85-96-success-in-sim-with-zero-demonstration-455a9044d353)).

### What already exists

| Project | What it covers | What it lacks |
| --- | --- | --- |
| [motorbridge](https://pypi.org/project/motorbridge/) | Rust core, Python SDK, CLI, WebSocket gateway; Damiao, RobStride, MyActuator, Hexfellow, HighTorque; unified scan; MIT license; releases every few days | No spec file, thermal model, bus budget or limit enforcement |
| [misa-actuator](https://github.com/takarakasai/misa-actuator) | Rust trait over RobStride, LK, Damiao, MyActuator; driver-agnostic TUI | Early, 0 stars, no limits layer |
| [cubemarsPyCAN](https://github.com/verlab/cubemarsPyCAN) | CubeMars MIT and servo, simulator, soak tests | One vendor |
| [ros2-arm-spec-lab](https://github.com/farouk15160/ros2-arm-spec-lab) | YAML arm spec, torque-speed envelope, thermal model, pass/fail report | Design-time only, no runtime guard |
| [robotbase-kit](https://pypi.org/project/robotbase-kit/) | Agent-operated ROS 2 + Gazebo sim with YAML scenarios | No actuator or bus layer |

The gap is clear: drivers are getting unified, but no project connects ratings, URDF and runtime enforcement.

## Feature evaluation

Fourteen of 24 candidate features make v1; two are cut outright. The pessimist column wins wherever an active project already does the job well.

| Feature | Optimist: why it gets used | Pessimist: why it won't | Verdict |
| --- | --- | --- | --- |
| Any-adapter transport (SocketCAN, slcan, PCAN, gs\_usb) | Kills vendor dongle lock-in, the reason CyberGearDashboard exists | motorbridge already does this well | Reuse |
| Multi-vendor bus scan | Everyone starts by asking "what is on this bus" | motorbridge already scans all its vendors | Reuse |
| Schema-driven Studio GUI (buttons generated per family) | Replaces five vendor tools with one; the demo that gets adoption | TUIs exist; a GUI is only valuable if it adds limits and plots | Build v1 |
| Verified enable (confirm mode and torque, not just a reply) | Directly fixes the silent MIT enable bug seen in the wild | Needs per-family verification logic | Build v1 |
| Firmware-versioned protocol profiles | Scaling constants change per firmware; wrong ones mean wrong torque | Hard to collect without owning every firmware | Build v1 |
| Parameter table read, diff against spec, write to flash | Makes config reviewable and repeatable across a team | Writing flash is risky; needs confirm and readback | Build v1 |
| Safe ID and bitrate wizard (one motor on bus) | ID collisions are a known Damiao setup trap | Rarely used after bring-up | v1.1 |
| Spec file plus URDF import | Single source of truth for every later check | Extra file to maintain; must stay short | Build v1 |
| Hold-torque check at named poses | Standing overheat is the most common leg complaint; one number explains it | Only as good as URDF masses; needs bench validation | Build v1 |
| Duty-profile thermal prediction | Answers "how long can I squat" before trying it | Datasheet thermal constants are unreliable | v1.1 |
| Bus load calculator | Tells you when to split buses before jitter shows up | Simple arithmetic people can do by hand | Build v1 |
| Supply and regen budget check | Regen misconfig is a proven way to kill drives | Most drives protect themselves; mainly helps batteries | Build v1 |
| Firmware limit readback before arming | Catches the flashed-wrong-config case that software clamps miss | Not every family exposes limits over CAN | Build v1 |
| Runtime gateway as sole bus writer | Makes every other limit enforceable, for humans and agents | Adds latency; must be rock solid or nobody runs it | Build v1 |
| Per-motor I²t | Industry-standard thermal guard needing no thermal constants | Firmware may already do it; double limiting can confuse | Build v1 |
| Watchdog plus fault policy (damp, zero torque, hold) | Leg robots need a slow sag, not a drop | Behavior differs per family; must be tested each time | Build v1 |
| Structured trip log | Turns a clamp into a lesson for the human or the agent | Useless without the gateway | Build v1 |
| MCP server for coding agents | Agents struggle most at the hardware connection step | Crowded space (ROS-MCP, robotbase); must lead with limits | Build v1 |
| Sim twin (vcan emulator with thermal model) | Agents can hit thermal failure safely before hardware | Physics coupling is a big job; start with the emulator only | v1.1 |
| Browser version over WebSerial and ESP32 bridge | Zero install for classes and clubs | Duplicates the desktop GUI; timing limits at 1 kHz | Later |
| Capture-replay CI for contributed profiles | Lets strangers add motors without breaking others | Only matters once contributors show up | v1.1 |
| ros2\_control hardware interface | Most labs run ROS 2; lowers adoption cost | Big surface; wait until the gateway API is stable | Later |
| EtherCAT support | Covers industrial drives | Different users and market; no overlap with QDD builders | Cut |
| Functional safety certification claims | Would open industrial users | Impossible for an open-source student project; liability risk | Cut |

Reuse means wrapping motorbridge as a driver backend rather than writing those codecs again. Your own YAML schema codec covers families it lacks, starting with CubeMars MIT mode.

## v1 scope

v1 ships when your humanoid runs through the gateway and one outside team runs the Studio. Everything below is ordered by what unblocks the next thing.

**Must have (v1)**

1. Driver backends: motorbridge for RobStride and Damiao, own YAML codec for CubeMars MIT.
2. Verified enable and firmware-versioned profiles.
3. Spec file, URDF import, effective-limit compiler.
4. Static checks: hold torque, bus load, supply and regen budget, mismatch report.
5. Gateway: sole writer, clamps, I²t, watchdog, fault policy, trip log, firmware readback on arm.
6. Studio GUI: scan, enable, zero, mode, jog, parameters with spec diff, live plots.
7. MCP server exposing status, trip log, thermal headroom and checks.

**Next (v1.1)**

- Duty-profile thermal prediction, once bench data exists.
- vcan emulator with thermal model for agent practice.
- Safe ID and bitrate wizard.
- Capture-replay CI for outside contributions.

**Later:** browser version, ros2\_control interface.

**Cut:** EtherCAT, any safety certification claim. The README states plainly that this protects equipment and is not a human safety system.

## Architecture

One gateway process owns every bus, and everything else talks to it. The spec compiler turns your spec into `limits.json`; clients send commands; the gateway enforces, encodes and sends.

&#91;embedded content: harness architecture · spec feeds limits, clients feed commands, one gateway writes the bus\]

The split by language follows the job: Rust for the hot loop, Python for spec math and the MCP server, TypeScript for the Studio.

Four data contracts hold the system together:

- **`robot_hw.yaml`:** human-written spec, validated by a pydantic model.
- **`profiles/<family>/<model>_<fw>.yaml`:** protocol data per motor family and firmware, each field citing its manual section.
- **`limits.json`:** compiled effective limits the gateway loads at arm time; never hand-edited.
- **Trip log:** one JSON line per clamp or trip with time, joint, limit, commanded, allowed and action.

## Sprint plan

Six milestones over about six weeks at part-time hours. A milestone is done only when every gate test passes; a failed gate means fix or cut scope, never push forward. Write the gate tests first and let agents build against them.

### M0: Foundations (days 1 to 2)

- [ ] Monorepo with `driver/` (Rust), `spec/` (Python), `studio/` (TypeScript), `mcp/` (Python), `profiles/`, `captures/`
- [ ] Apache-2.0 license, CONTRIBUTING.md, CI running lint and unit tests on every PR
- [ ] Spec schema v0 as a pydantic model plus JSON Schema export
- [ ] Agent files from the sections below committed
- [ ] vcan0 test rig script that works locally and in CI

| Gate test | Pass |
| --- | --- |
| CI on a trivial PR | Green in under 5 minutes |
| `make vcan-up && make test` on a clean Ubuntu 24.04 machine | Passes with no manual steps |

### M1: Driver layer (week 1)

- [ ] `Backend` trait: scan, enable, disable, zero, set\_mode, command, read\_params, write\_params, feedback stream
- [ ] motorbridge backend for RobStride and Damiao
- [ ] YAML schema codec and a CubeMars MIT profile, keyed by model and firmware
- [ ] Verified enable: confirm the motor reports enabled state or torque response, else raise
- [ ] Replay transport that plays a `candump` log as a fake bus
- [ ] ESP32 TWAI serial bridge firmware (your ESP32 plus a transceiver chip)

| Gate test | Pass |
| --- | --- |
| Codec round-trip property test, every field | Error of 1 LSB or less |
| Decode a 10-minute real capture per family | 100% of frames decoded, 0 unknown |
| Scan a mixed-vendor bus | Every motor found with correct ID and family, 0 false positives |
| Enable a motor with a deliberately wrong mode | Verified enable raises, never reports success |
| ESP32 bridge at 1 kHz for 10 minutes | 0 dropped frames |

### M2: Spec and static checks (week 2)

- [ ] Spec loader with URDF import and joint-to-actuator mapping
- [ ] Actuator library entries for the motors you own, datasheet fields marked `unverified`
- [ ] Effective-limit compiler: minimum of actuator, URDF, user cap, transmission and profile scale; outputs `limits.json` for the gateway
- [ ] Checks: hold torque at named poses, bus load, supply and regen budget, CAN ID collisions, mismatch report
- [ ] `harness check robot_hw.yaml` CLI with human and JSON output

| Gate test | Pass |
| --- | --- |
| Suite of 20 deliberately broken configs | 100% flagged, each with the right error code |
| Known-good config | 0 findings |
| Hold torque at standing pose vs measured current × Kt on the robot | Within ±10% on every loaded joint |
| Bus load estimate vs measured `canbusload` | Within ±5 percentage points |

### M3: Gateway (week 3)

- [ ] Rust daemon owning every CAN interface; other processes lose write access through group permissions
- [ ] WebSocket and Unix-socket API: command, state stream, arm, disarm, trip log
- [ ] Enforcement: position, velocity and torque clamps, per-motor I²t, bus current budget, command rate limit
- [ ] Watchdog with per-joint fault policy: damp, zero torque or hold
- [ ] Arm sequence: firmware limit readback, temperature check, bus voltage check, encoder range check
- [ ] Trip log as JSON lines: time, joint, limit, commanded, allowed, action

| Gate test | Pass |
| --- | --- |
| Write to `can0` from a non-gateway process | Fails with permission denied |
| Added latency at 1 kHz, all motors | p99 of 0.5 ms or less |
| 10,000 injected over-limit commands | 100% handled per policy, each logged |
| Kill the command stream | Safe state within timeout plus one cycle |
| Bench: hold above continuous torque | I²t derates or trips with drive temp at least 10 °C below max |
| Firmware limit looser than spec | Arm refused with a named reason |

### M4: Studio (week 4)

- [ ] Web UI served by the gateway: bus scan, per-motor panel generated from the profile
- [ ] Enable, disable, zero, mode switch, jog sliders bounded by effective limits
- [ ] Parameter table with diff against spec; flash writes need confirmation and readback
- [ ] Live plots of position, velocity, torque, temperature and bus voltage; decoded faults in plain English
- [ ] Raw frame view next to the decoded view

| Gate test | Pass |
| --- | --- |
| Full bring-up of a new motor, no vendor software | Discover, zero, jog and read temperature succeed |
| Family-specific UI code | None; every control comes from the profile |
| Triton Droids member following README only | Bring-up in under 15 minutes |

### M5: Agent layer (week 5)

- [ ] MCP server: `bus_status`, `motor_status`, `trip_log`, `thermal_headroom`, `check_spec`, `run_on_emulator`
- [ ] vcan emulator that answers each profile and simulates heating with a first-order model
- [ ] Agent benchmark of 5 tasks, including a standing controller and a homing routine
- [ ] Demo video: agent writes a controller that would cook a knee, harness flags it, agent fixes it

| Gate test | Pass |
| --- | --- |
| Same user code on emulator and hardware | Only a config flag changes |
| Benchmark runs through the gateway | 0 limit violations reach the real bus |
| Success rate with harness vs without | Higher with harness on at least 4 of 5 tasks |

### M6: Harden and launch (week 6)

- [ ] 20 hours of humanoid operation through the gateway, spread across the week
- [ ] Capture-replay CI so contributed profiles must ship a real capture
- [ ] Docs site with quickstart, spec reference and "add a motor" guide
- [ ] Public release and posts where QDD builders gather

| Gate test | Pass |
| --- | --- |
| 20-hour runtime log | 0 unexplained faults, every trip logged |
| Outside contributor adds a family | Merged using docs only |
| Outside team on real hardware | At least 1 running within 2 weeks of launch |

## Root CLAUDE.md

Copy this to the repo root. It is terse on purpose: agents follow short rules better, and it matches the output style you want from them.

```markdown
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
```

## Other agent files

Three subagents, three commands, one hook script and two package files. The hook is the part that matters most: it enforces the hardware and protected-path rules even when the agent forgets them.

### .claude/agents/safety-reviewer.md

```markdown
---
name: safety-reviewer
description: Reviews any diff touching driver/src/gateway, limits, checks, or profiles. Use before every merge in those paths.
tools: Read, Grep, Glob, Bash
---
You review for equipment damage risk. You do not write code.
Check:
- any limit, default, tolerance loosened? list each with file:line
- any path where a command reaches the bus without passing enforce/?
- unit conversions: SI inside, vendor units only in profiles/codec
- new check has both violating and passing test
- fault paths: what happens on timeout, decode error, NaN, bus-off
Output: BLOCK / OK, then findings as bullets. BLOCK on any bypass or loosened limit.
```

### .claude/agents/profile-author.md

```markdown
---
name: profile-author
description: Writes or updates a YAML protocol profile for a motor family from a manual and a capture.
tools: Read, Write, Edit, Grep, Glob, Bash
---
Inputs: manual excerpt or link text, captures/<family>/<model>_<fw>.log.
Steps:
1. Draft profiles/<family>/<model>_<fw>.yaml. Every field needs a manual section ref in a comment.
2. Run make replay CAPTURE=... PROFILE=...
3. Target: 100% frames decoded, 0 unknown, round-trip within 1 LSB.
4. Ambiguity in manual -> write it under open_questions: in the profile. Never guess.
Never edit captures/. Never touch hardware.
```

### .claude/agents/gate-runner.md

```markdown
---
name: gate-runner
description: Runs milestone gate tests and reports pass/fail per gate. Use at end of each task and before claiming a milestone done.
tools: Read, Bash, Grep
---
Run: make test, then make gates M=$MILESTONE.
Report table: gate | result | evidence (numbers, log path).
Hardware gates: mark NEEDS-HUMAN with the exact command for the human to run.
Never edit tests to make them pass.
```

### .claude/commands/milestone.md

```markdown
Start milestone $ARGUMENTS.
1. Read tests/gates/m$ARGUMENTS/ and the sprint section of docs/plan.md.
2. List tasks not done. Propose order. Wait for OK.
3. Per task: plan, implement, test, then gate-runner.
4. Touching gateway/limits/profiles -> safety-reviewer before commit.
```

### .claude/commands/add-motor.md

```markdown
Add motor $ARGUMENTS (format: family/model/firmware).
1. Check captures/ has a real capture. None -> stop, ask human for one.
2. Use profile-author subagent.
3. Add actuator library entry, rating fields marked unverified.
4. make replay + make test. Report decode coverage.
```

### .claude/commands/check-spec.md

```markdown
Run harness check on $ARGUMENTS. Explain each finding in one line with the fix.
Do not change the spec file unless asked.
```

### .claude/settings.json

```json
{
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [{ "type": "command", "command": "python3 .claude/hooks/guard.py" }] },
      { "matcher": "Edit|Write|MultiEdit", "hooks": [{ "type": "command", "command": "python3 .claude/hooks/guard.py" }] }
    ],
    "PostToolUse": [
      { "matcher": "Edit|Write|MultiEdit", "hooks": [{ "type": "command", "command": "make -s quick-test" }] }
    ]
  }
}
```

### .claude/hooks/guard.py

```python
#!/usr/bin/env python3
"""Blocks hardware access and edits to protected paths. Exit 2 = block."""
import json, re, sys

ev = json.load(sys.stdin)
tool = ev.get("tool_name", "")
inp = ev.get("tool_input", {})

PROTECTED = ("driver/src/gateway/enforce/", "captures/", "tests/gates/")
HW = [r"\bcan\d\b", r"ip link set", r"/dev/tty(USB|ACM)", r"slcand", r"cansend\s+can", r"--channel\s+can\d"]

if tool == "Bash":
    cmd = inp.get("command", "")
    if any(re.search(p, cmd) for p in HW) and "vcan" not in cmd:
        print("BLOCKED: real hardware. Use vcan/replay/emulator, or ask the human to run it.", file=sys.stderr)
        sys.exit(2)
else:
    path = inp.get("file_path", "")
    if any(p in path for p in PROTECTED):
        print(f"BLOCKED: {path} is protected. Propose the change in the PR description.", file=sys.stderr)
        sys.exit(2)
sys.exit(0)
```

### driver/CLAUDE.md

```markdown
Rust. Hot path = gateway loop. No allocation, no locks held across await, no panics in loop.
Every command: decode -> enforce -> encode -> send. enforce/ is human-owned.
Latency budget 0.5 ms p99 added. Bench with cargo bench before/after any loop change.
Backend trait is the only vendor seam.
```

### spec/CLAUDE.md

```markdown
Python 3.11, pydantic v2, numpy. Pure functions. No I/O in checks.
Every check returns Finding(code, joint, severity, message, fix).
New check -> add a broken config under tests/fixtures/broken/ and a passing case.
Gravity torque via URDF masses/COMs. Show your formula in the docstring.
```

## Risks and open questions

| Risk | Fallback |
| --- | --- |
| motorbridge changes its API or stalls | Pin a version; the Backend trait lets you swap in your own codec per family |
| Gateway latency misses 0.5 ms on the Jetson Nano | Run the gateway on the laptop for v1; move enforcement into the ESP32 bridge later |
| Datasheet thermal constants are wrong | I²t needs none; mark model-based predictions unverified until bench-fitted |
| Double limiting (firmware plus gateway) confuses users | Trip log names which layer acted; Studio shows both limits side by side |
| Scope creep into sim physics | Emulator only in v1; physics coupling waits for outside demand |
| An agent-written enforcement bug burns a motor | enforce/ is human-owned, hook-protected, and reviewed line by line |

Open questions:

- [ ] Which actuator families and firmware versions are on your humanoid today?
- [ ] Does motorbridge's maintainer accept a limits hook upstream, or does it stay a separate layer?
- [ ] Which outside team commits to trying the Studio in week 4?
