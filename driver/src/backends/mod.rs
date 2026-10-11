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
//! Backends:
//! - `robstride`, `damiao`: wrap the `motorbridge` project's vendor
//!   crates (git-pinned, see driver/Cargo.toml) -- both are "Reuse"
//!   targets per the sprint plan's feature evaluation.
//! - `yaml_codec` (CubeMars MIT mode): not yet started.
//!
//! A `mock` backend (test-only) exists so the `Backend` trait's contract
//! -- especially verified-enable -- can be tested without any real
//! protocol data at all.

pub mod damiao;
#[cfg(test)]
pub mod mock;
pub mod robstride;
