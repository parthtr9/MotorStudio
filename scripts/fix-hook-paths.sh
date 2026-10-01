#!/usr/bin/env bash
# Rewrites the absolute paths in .claude/settings.json's hook commands to
# match THIS machine's actual clone location. Run this once after cloning
# the repo to a new machine/path -- the hook commands are absolute (not
# relative) on purpose, see MACHINE_SETUP.md for why, but that means they
# still need to match wherever this clone actually lives.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SETTINGS="$REPO_ROOT/.claude/settings.json"

if [[ ! -f "$SETTINGS" ]]; then
  echo "fix-hook-paths: $SETTINGS not found" >&2
  exit 1
fi

python3 - "$SETTINGS" "$REPO_ROOT" <<'PYEOF'
import json
import re
import sys

settings_path, repo_root = sys.argv[1], sys.argv[2]
with open(settings_path) as f:
    text = f.read()

# Replace any absolute path that ends in /.claude/hooks/guard.py, and any
# `make -C <path>` target, with this machine's actual repo root. Validates
# as JSON afterwards so a bad replace can't silently corrupt the file.
new_text = re.sub(r"[^\s\"]*/\.claude/hooks/guard\.py", f"{repo_root}/.claude/hooks/guard.py", text)
new_text = re.sub(r"make -s -C [^\s\"]*", f"make -s -C {repo_root}", new_text)

json.loads(new_text)  # raises if this broke the JSON

if new_text != text:
    with open(settings_path, "w") as f:
        f.write(new_text)
    print(f"fix-hook-paths: updated {settings_path} to use {repo_root}")
else:
    print("fix-hook-paths: already correct, no change needed")
PYEOF
