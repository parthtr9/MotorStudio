//! Replay transport: plays back a `candump -L` log file as a fake bus.
//!
//! The `candump -L` line format is a standard Linux SocketCAN / can-utils
//! log format, documented by that project -- not vendor-specific, so
//! parsing it is not a protocol-guessing concern the way decoding a
//! motor's own frame payloads would be. Format per line:
//!
//! ```text
//! (1699999999.123456) can0 123#DEADBEEF
//! (1699999999.234567) can0 18FD0001#0102030405060708
//! (1699999999.345678) can0 123#R          <- remote frame, no data
//! ```
//!
//! `(timestamp) interface id#data[#flags]`. `id` is hex, 3 digits for an
//! 11-bit standard frame or 8 digits for a 29-bit extended frame (the
//! convention candump itself uses to tell them apart on read-back). This
//! module only parses that envelope; it does not interpret payload bytes
//! -- that's a `Backend`'s job, using a cited profile.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayFrame {
    pub timestamp_s: f64,
    pub interface: String,
    pub can_id: u32,
    pub extended: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayParseError {
    pub line_no: usize,
    pub detail: String,
}

impl fmt::Display for ReplayParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line_no, self.detail)
    }
}

impl std::error::Error for ReplayParseError {}

fn parse_line(line: &str, line_no: usize) -> Result<Option<ReplayFrame>, ReplayParseError> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }

    let err = |detail: &str| ReplayParseError {
        line_no,
        detail: detail.to_string(),
    };

    let rest = line
        .strip_prefix('(')
        .ok_or_else(|| err("expected line to start with '('"))?;
    let (ts_str, rest) = rest
        .split_once(')')
        .ok_or_else(|| err("missing closing ')' after timestamp"))?;
    let timestamp_s: f64 = ts_str
        .trim()
        .parse()
        .map_err(|_| err("timestamp is not a valid number"))?;

    let rest = rest.trim();
    let (interface, rest) = rest
        .split_once(' ')
        .ok_or_else(|| err("missing interface name after timestamp"))?;

    let mut parts = rest.trim().split('#');
    let id_str = parts.next().ok_or_else(|| err("missing CAN id"))?;
    let data_str = parts.next().unwrap_or("");

    if id_str.is_empty() {
        return Err(err("empty CAN id field"));
    }
    let extended = id_str.len() > 3;
    let can_id = u32::from_str_radix(id_str, 16).map_err(|_| err("CAN id is not valid hex"))?;

    if data_str == "R" {
        // Remote frame: no data payload.
        return Ok(Some(ReplayFrame {
            timestamp_s,
            interface: interface.to_string(),
            can_id,
            extended,
            data: Vec::new(),
        }));
    }

    if data_str.len() % 2 != 0 {
        return Err(err("data field has an odd number of hex digits"));
    }
    let mut data = Vec::with_capacity(data_str.len() / 2);
    let mut chars = data_str.chars();
    while let (Some(hi), Some(lo)) = (chars.next(), chars.next()) {
        let byte_str: String = [hi, lo].iter().collect();
        let byte = u8::from_str_radix(&byte_str, 16).map_err(|_| err("data is not valid hex"))?;
        data.push(byte);
    }
    if data.len() > 8 {
        return Err(err(
            "data field has more than 8 bytes, not a valid CAN frame",
        ));
    }

    Ok(Some(ReplayFrame {
        timestamp_s,
        interface: interface.to_string(),
        can_id,
        extended,
        data,
    }))
}

/// A loaded, ordered sequence of frames from a `candump -L` capture.
#[derive(Debug, Clone, Default)]
pub struct ReplayLog {
    pub frames: Vec<ReplayFrame>,
}

impl ReplayLog {
    pub fn parse(contents: &str) -> Result<Self, ReplayParseError> {
        let mut frames = Vec::new();
        for (i, line) in contents.lines().enumerate() {
            if let Some(frame) = parse_line(line, i + 1)? {
                frames.push(frame);
            }
        }
        Ok(ReplayLog { frames })
    }

    pub fn frames_for_interface<'a>(
        &'a self,
        interface: &'a str,
    ) -> impl Iterator<Item = &'a ReplayFrame> {
        self.frames.iter().filter(move |f| f.interface == interface)
    }
}

impl FromStr for ReplayLog {
    type Err = ReplayParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ReplayLog::parse(s)
    }
}

/// Sequential cursor over a loaded log, for a fake-bus transport that
/// hands frames to a `Backend` one at a time in capture order.
#[derive(Debug)]
pub struct ReplaySource {
    log: ReplayLog,
    cursor: usize,
}

impl ReplaySource {
    pub fn new(log: ReplayLog) -> Self {
        ReplaySource { log, cursor: 0 }
    }

    pub fn next_frame(&mut self) -> Option<&ReplayFrame> {
        let frame = self.log.frames.get(self.cursor);
        if frame.is_some() {
            self.cursor += 1;
        }
        frame
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    pub fn remaining(&self) -> usize {
        self.log.frames.len().saturating_sub(self.cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_frame() {
        let log = ReplayLog::parse("(1699999999.123456) can0 123#DEADBEEF").unwrap();
        assert_eq!(log.frames.len(), 1);
        let f = &log.frames[0];
        assert_eq!(f.timestamp_s, 1699999999.123456);
        assert_eq!(f.interface, "can0");
        assert_eq!(f.can_id, 0x123);
        assert!(!f.extended);
        assert_eq!(f.data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parses_extended_frame() {
        let log = ReplayLog::parse("(1699999999.234567) can0 18FD0001#0102030405060708").unwrap();
        let f = &log.frames[0];
        assert_eq!(f.can_id, 0x18FD0001);
        assert!(f.extended);
        assert_eq!(f.data.len(), 8);
    }

    #[test]
    fn parses_remote_frame_with_no_data() {
        let log = ReplayLog::parse("(1699999999.1) can0 123#R").unwrap();
        assert_eq!(log.frames[0].data.len(), 0);
    }

    #[test]
    fn skips_blank_lines() {
        let log = ReplayLog::parse("(1.0) can0 123#AA\n\n(2.0) can0 124#BB\n").unwrap();
        assert_eq!(log.frames.len(), 2);
    }

    #[test]
    fn filters_by_interface() {
        let log =
            ReplayLog::parse("(1.0) can0 123#AA\n(2.0) can1 124#BB\n(3.0) can0 125#CC").unwrap();
        let can0: Vec<_> = log.frames_for_interface("can0").collect();
        assert_eq!(can0.len(), 2);
    }

    #[test]
    fn rejects_missing_parens() {
        let err = ReplayLog::parse("1699999999.123456 can0 123#DEADBEEF").unwrap_err();
        assert_eq!(err.line_no, 1);
    }

    #[test]
    fn rejects_odd_length_data() {
        let err = ReplayLog::parse("(1.0) can0 123#ABC").unwrap_err();
        assert!(err.detail.contains("odd number"));
    }

    #[test]
    fn rejects_more_than_8_data_bytes() {
        let err = ReplayLog::parse("(1.0) can0 123#0102030405060708090A").unwrap_err();
        assert!(err.detail.contains("more than 8 bytes"));
    }

    #[test]
    fn rejects_bad_hex_id() {
        let err = ReplayLog::parse("(1.0) can0 ZZZ#AA").unwrap_err();
        assert!(err.detail.contains("not valid hex"));
    }

    #[test]
    fn source_cursor_advances_and_reports_remaining() {
        let log = ReplayLog::parse("(1.0) can0 1#AA\n(2.0) can0 2#BB").unwrap();
        let mut source = ReplaySource::new(log);
        assert_eq!(source.remaining(), 2);
        assert_eq!(source.next_frame().unwrap().can_id, 1);
        assert_eq!(source.remaining(), 1);
        assert_eq!(source.next_frame().unwrap().can_id, 2);
        assert!(source.next_frame().is_none());
        source.reset();
        assert_eq!(source.remaining(), 2);
    }
}
