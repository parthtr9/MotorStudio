"""Reads the gateway's trip log (JSON lines, one TripLogEntry per line --
see driver/src/gateway/trip_log.rs for the writer and
driver/src/gateway/types.rs for the exact field shape this must match).

No gateway with a network API exists yet (see driver/src/gateway/
daemon.rs's module doc), so there is no live trip log file in normal
operation today. This module only reads whatever file it's pointed at;
it doesn't know or care whether a real gateway produced it.
"""
from __future__ import annotations

import json
from pathlib import Path


def read_trip_log(path: str | Path, limit: int | None = None) -> list[dict]:
    """Returns parsed trip log entries in file order (oldest first). If
    `limit` is set, returns only the last `limit` entries (most recent
    trips), since that's almost always what a caller actually wants.
    Returns an empty list if the file doesn't exist yet -- an empty
    trip log is a normal state (nothing has tripped), not an error.
    """
    file_path = Path(path)
    if not file_path.exists():
        return []
    entries = []
    for line in file_path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        entries.append(json.loads(line))
    if limit is not None:
        entries = entries[-limit:]
    return entries
