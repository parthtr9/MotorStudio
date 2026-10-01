//! `Backend` implementations, one per motor family/wire strategy.
//!
//! Per root `CLAUDE.md`: "Unknown frame or param -> log + reject. Never
//! guess." No backend here encodes/decodes a real RobStride (or any
//! other family's) frame yet, because no `profiles/<family>/*.yaml` has
//! been finalized against a confirmed firmware version and a real
//! capture -- see `profiles/robstride/OPEN_QUESTIONS.md`. Writing a
//! backend against unconfirmed protocol data would be exactly the kind
//! of guessing that rule forbids.
//!
//! Planned backends (M1, once unblocked):
//! - `motorbridge`: wraps the existing `motorbridge` crate/CLI for
//!   RobStride and Damiao (per the sprint plan's feature evaluation,
//!   these are "Reuse" targets, not a from-scratch codec).
//! - `yaml_codec`: a from-scratch codec driven by a `ProtocolProfile`
//!   (the Python-side schema in `spec/harness/profile_schema.py` has a
//!   Rust-side equivalent to come), starting with CubeMars MIT mode.
//!
//! A `mock` backend (test-only) exists so the `Backend` trait's contract
//! -- especially verified-enable -- can be tested without any real
//! protocol data at all.

#[cfg(test)]
pub mod mock;
