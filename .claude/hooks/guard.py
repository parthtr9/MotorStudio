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
