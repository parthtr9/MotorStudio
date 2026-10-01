//! Append-only JSON-lines writer for the trip log. This is I/O
//! formatting, not an enforcement decision -- it just durably records
//! whatever `EnforceDecision`s `enforce/` already made. One
//! `TripLogEntry` per line, per the architecture doc.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::TripLogEntry;

pub struct TripLog {
    file: File,
}

impl TripLog {
    /// Opens (creating if needed) a trip log file in append mode. Never
    /// truncates an existing log -- trip history is an audit trail.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(TripLog { file })
    }

    pub fn append(&mut self, entry: &TripLogEntry) -> io::Result<()> {
        let mut line = serde_json::to_string(entry)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        line.push('\n');
        self.file.write_all(line.as_bytes())?;
        self.file.flush()
    }
}

/// Reads back a trip log file (mirrors mcp/mcp_server/trip_log_reader.py's
/// read_trip_log -- same JSON-lines format, same "missing file is empty,
/// not an error" and "limit means most recent N" semantics, kept in sync
/// by hand since one side is Rust and the other Python).
pub fn read_entries(path: impl AsRef<Path>, limit: Option<usize>) -> io::Result<Vec<TripLogEntry>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(path)?;
    let mut entries = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let entry: TripLogEntry = serde_json::from_str(line)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        entries.push(entry);
    }
    if let Some(limit) = limit
        && entries.len() > limit
    {
        entries = entries.split_off(entries.len() - limit);
    }
    Ok(entries)
}

/// Current time as Unix milliseconds, for `TripLogEntry::time_unix_ms`.
/// Not a safety decision -- a timestamp helper.
pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::types::EnforceAction;
    use std::io::{BufRead, BufReader};

    fn sample_entry(commanded: f64) -> TripLogEntry {
        TripLogEntry {
            time_unix_ms: now_unix_ms(),
            joint: "knee".to_string(),
            limit: "max_torque_nm".to_string(),
            commanded,
            allowed: 9.0,
            action: EnforceAction::ClampedTorque,
        }
    }

    #[test]
    fn appends_one_json_line_per_entry() {
        let dir = std::env::temp_dir().join(format!("harness-triplog-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("trip.jsonl");
        let _ = std::fs::remove_file(&path);

        let mut log = TripLog::open(&path).unwrap();
        log.append(&sample_entry(15.0)).unwrap();
        log.append(&sample_entry(20.0)).unwrap();
        drop(log);

        let reader = BufReader::new(File::open(&path).unwrap());
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
        assert_eq!(lines.len(), 2);
        let first: TripLogEntry = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(first.commanded, 15.0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reopening_appends_rather_than_truncates() {
        let dir =
            std::env::temp_dir().join(format!("harness-triplog-test2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("trip.jsonl");
        let _ = std::fs::remove_file(&path);

        TripLog::open(&path)
            .unwrap()
            .append(&sample_entry(1.0))
            .unwrap();
        TripLog::open(&path)
            .unwrap()
            .append(&sample_entry(2.0))
            .unwrap();

        let reader = BufReader::new(File::open(&path).unwrap());
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
        assert_eq!(lines.len(), 2, "reopening must append, not truncate");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_entries_on_missing_file_returns_empty_not_error() {
        let entries = read_entries("/no/such/path/trip.jsonl", None).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn read_entries_round_trips_and_respects_limit() {
        let dir =
            std::env::temp_dir().join(format!("harness-triplog-test3-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("trip.jsonl");
        let _ = std::fs::remove_file(&path);

        let mut log = TripLog::open(&path).unwrap();
        for i in 0..5 {
            log.append(&sample_entry(i as f64)).unwrap();
        }
        drop(log);

        let all = read_entries(&path, None).unwrap();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0].commanded, 0.0);

        let last_two = read_entries(&path, Some(2)).unwrap();
        assert_eq!(last_two.len(), 2);
        assert_eq!(last_two[0].commanded, 3.0);
        assert_eq!(last_two[1].commanded, 4.0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
