//! Transports move raw CAN frames; they know nothing about motor
//! protocols. A `Backend` sits on top of one of these and a profile to
//! interpret payloads.

pub mod replay;
