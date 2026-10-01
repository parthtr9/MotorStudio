import json

from mcp_server.trip_log_reader import read_trip_log


def test_missing_file_returns_empty_list(tmp_path):
    assert read_trip_log(tmp_path / "no_such.jsonl") == []


def test_reads_entries_in_order(tmp_path):
    path = tmp_path / "trip.jsonl"
    entries = [{"joint": "knee", "n": i} for i in range(3)]
    path.write_text("\n".join(json.dumps(e) for e in entries) + "\n")
    result = read_trip_log(path)
    assert result == entries


def test_limit_returns_most_recent_entries(tmp_path):
    path = tmp_path / "trip.jsonl"
    entries = [{"n": i} for i in range(5)]
    path.write_text("\n".join(json.dumps(e) for e in entries) + "\n")
    result = read_trip_log(path, limit=2)
    assert result == [{"n": 3}, {"n": 4}]


def test_skips_blank_lines(tmp_path):
    path = tmp_path / "trip.jsonl"
    path.write_text('{"n": 1}\n\n{"n": 2}\n')
    result = read_trip_log(path)
    assert result == [{"n": 1}, {"n": 2}]
