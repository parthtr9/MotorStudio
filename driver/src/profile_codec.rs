//! Generic, profile-driven CAN frame field codec.
//!
//! Encodes/decodes exactly what `spec/harness/profile_schema.py`'s
//! `FrameField` defines: an unsigned integer read over `byte_offset`/
//! `byte_length` bytes in `byte_order`, affine-mapped to a physical
//! value via `scale`/`offset` (`raw * scale + offset`) -- the same
//! pattern RobStride's and Damiao's own vendor crates already use for
//! their real frames (`uint_to_float`-style conversions, see
//! `backends/robstride.rs`/`backends/damiao.rs`).
//!
//! This module has NO knowledge of any vendor family, is not a
//! `Backend` (see `backend.rs`), and is not registered in
//! `backends/mod.rs`. Per that module's doc, no backend may
//! encode/decode a real frame until a finalized, manual-cited
//! `profiles/<family>/*.yaml` exists. This is the generic machinery a
//! future `yaml_codec` `Backend` (CubeMars MIT mode) would use once
//! that profile lands; by itself it proves nothing about any real
//! motor's wire format -- every test below uses synthetic, made-up
//! field definitions, same posture as `spec/tests/test_profile_schema.py`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    Big,
    Little,
}

/// Mirrors one `FrameField` from the profile schema.
/// `scale`/`offset`/the decoded value are all `f64`, which only
/// represents integers exactly up to 2^53. For `byte_length` of 7 or 8
/// (56/64-bit raw range), `max_raw()` and any raw value above 2^53 are
/// NOT guaranteed to round-trip bit-exactly through `decode`/`encode` --
/// only approximately, within `f64`'s precision. Every real field this
/// codec exists for (position/velocity/torque/temperature, 1-4 bytes
/// typically, per RobStride's/Damiao's own frames) is far below that
/// boundary; byte_length 1-6 round-trips every raw value exactly.
#[derive(Debug, Clone, Copy)]
pub struct FieldCodec {
    pub byte_offset: usize,
    pub byte_length: usize,
    pub byte_order: ByteOrder,
    pub scale: f64,
    pub offset: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodecError {
    FieldOutOfFrame {
        byte_offset: usize,
        byte_length: usize,
    },
    ValueOutOfRange {
        value: f64,
        min: f64,
        max: f64,
    },
}

impl FieldCodec {
    /// The largest raw unsigned integer this field's byte_length can hold.
    pub fn max_raw(&self) -> u64 {
        if self.byte_length >= 8 {
            u64::MAX
        } else {
            (1u64 << (self.byte_length * 8)) - 1
        }
    }

    /// The physical value range this field can represent, per its
    /// current scale/offset (`[offset, max_raw*scale + offset]` when
    /// scale is positive).
    pub fn value_range(&self) -> (f64, f64) {
        let max_value = self.max_raw() as f64 * self.scale + self.offset;
        if self.scale >= 0.0 {
            (self.offset, max_value)
        } else {
            (max_value, self.offset)
        }
    }

    fn check_bounds(&self) -> Result<(), CodecError> {
        if self.byte_length == 0 || self.byte_offset + self.byte_length > 8 {
            return Err(CodecError::FieldOutOfFrame {
                byte_offset: self.byte_offset,
                byte_length: self.byte_length,
            });
        }
        Ok(())
    }

    fn read_raw(&self, frame: &[u8; 8]) -> u64 {
        let bytes = &frame[self.byte_offset..self.byte_offset + self.byte_length];
        let mut raw = 0u64;
        match self.byte_order {
            ByteOrder::Big => {
                for &b in bytes {
                    raw = (raw << 8) | b as u64;
                }
            }
            ByteOrder::Little => {
                for &b in bytes.iter().rev() {
                    raw = (raw << 8) | b as u64;
                }
            }
        }
        raw
    }

    fn write_raw(&self, raw: u64, frame: &mut [u8; 8]) {
        let bytes = &mut frame[self.byte_offset..self.byte_offset + self.byte_length];
        match self.byte_order {
            ByteOrder::Big => {
                for (i, b) in bytes.iter_mut().rev().enumerate() {
                    *b = (raw >> (8 * i)) as u8;
                }
            }
            ByteOrder::Little => {
                for (i, b) in bytes.iter_mut().enumerate() {
                    *b = (raw >> (8 * i)) as u8;
                }
            }
        }
    }

    /// Reads this field out of a frame's 8 data bytes.
    pub fn decode(&self, frame: &[u8; 8]) -> Result<f64, CodecError> {
        self.check_bounds()?;
        Ok(self.read_raw(frame) as f64 * self.scale + self.offset)
    }

    /// Writes `value` into this field's bytes of `frame`, leaving every
    /// other byte untouched. Rejects (does not clamp) a value outside
    /// what `byte_length` can represent -- per root CLAUDE.md, "never
    /// guess" extends to never silently loosening what gets sent.
    pub fn encode(&self, value: f64, frame: &mut [u8; 8]) -> Result<(), CodecError> {
        self.check_bounds()?;
        let raw_f = ((value - self.offset) / self.scale).round();
        // Compare the ACTUAL resulting integer against max_raw, not
        // max_raw re-widened to f64 -- for byte_length 7/8, max_raw
        // itself isn't exactly representable as f64 (see FieldCodec's
        // doc), so comparing float-to-float here would let a rounded-up
        // out-of-range value slip through and get silently truncated by
        // write_raw's u8 casts instead of rejected.
        if raw_f.is_nan() || raw_f < 0.0 || raw_f > u64::MAX as f64 {
            let (min, max) = self.value_range();
            return Err(CodecError::ValueOutOfRange { value, min, max });
        }
        let raw = raw_f as u64;
        if raw > self.max_raw() {
            let (min, max) = self.value_range();
            return Err(CodecError::ValueOutOfRange { value, min, max });
        }
        self.write_raw(raw, frame);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(byte_offset: usize, byte_length: usize, byte_order: ByteOrder) -> FieldCodec {
        FieldCodec {
            byte_offset,
            byte_length,
            byte_order,
            scale: 1.0,
            offset: 0.0,
        }
    }

    #[test]
    fn big_endian_round_trips_raw_integer_exactly() {
        let f = field(2, 2, ByteOrder::Big);
        let mut frame = [0u8; 8];
        for raw in [0u64, 1, 0x1234, 0xFFFF] {
            f.encode(raw as f64, &mut frame).unwrap();
            assert_eq!(f.decode(&frame).unwrap() as u64, raw);
        }
    }

    #[test]
    fn little_endian_round_trips_raw_integer_exactly() {
        let f = field(2, 2, ByteOrder::Little);
        let mut frame = [0u8; 8];
        for raw in [0u64, 1, 0x1234, 0xFFFF] {
            f.encode(raw as f64, &mut frame).unwrap();
            assert_eq!(f.decode(&frame).unwrap() as u64, raw);
        }
    }

    #[test]
    fn big_and_little_endian_produce_different_bytes_for_the_same_value() {
        let big = field(0, 2, ByteOrder::Big);
        let little = field(0, 2, ByteOrder::Little);
        let mut big_frame = [0u8; 8];
        let mut little_frame = [0u8; 8];
        big.encode(0x1234_u32 as f64, &mut big_frame).unwrap();
        little.encode(0x1234_u32 as f64, &mut little_frame).unwrap();
        assert_eq!(big_frame[0..2], [0x12, 0x34]);
        assert_eq!(little_frame[0..2], [0x34, 0x12]);
    }

    #[test]
    fn affine_scale_and_offset_round_trip_within_one_lsb() {
        // Mirrors RobStride's/Damiao's own uint_to_float pattern: a
        // signed physical range from an unsigned raw integer.
        let f = FieldCodec {
            byte_offset: 0,
            byte_length: 2,
            byte_order: ByteOrder::Big,
            scale: 25.0 / 65535.0, // [-12.5, 12.5] rad over a u16
            offset: -12.5,
        };
        let mut frame = [0u8; 8];
        for value in [-12.5, -6.3, 0.0, 1.6, 12.5] {
            f.encode(value, &mut frame).unwrap();
            let decoded = f.decode(&frame).unwrap();
            assert!(
                (decoded - value).abs() <= f.scale,
                "value={value} decoded={decoded} scale={}",
                f.scale
            );
        }
    }

    #[test]
    fn every_byte_length_up_to_6_round_trips_its_max_raw_value_exactly() {
        // 6 bytes = 48 bits, comfortably inside f64's 53-bit exact
        // integer range -- see FieldCodec's doc for why 7/8 aren't
        // tested this way.
        for byte_length in 1..=6usize {
            let f = field(0, byte_length, ByteOrder::Big);
            let mut frame = [0u8; 8];
            let max_raw = f.max_raw();
            f.encode(max_raw as f64, &mut frame).unwrap();
            assert_eq!(f.decode(&frame).unwrap() as u64, max_raw);
        }
    }

    #[test]
    fn byte_length_7_and_8_round_trip_a_value_near_their_max_within_f64_precision() {
        // Not max_raw itself -- see the next test for why that's
        // correctly rejected, not round-trippable, at this width.
        for byte_length in [7usize, 8] {
            let f = field(0, byte_length, ByteOrder::Big);
            let mut frame = [0u8; 8];
            let near_max = f.max_raw() - 100_000;
            f.encode(near_max as f64, &mut frame).unwrap();
            let decoded = f.decode(&frame).unwrap();
            let relative_error = (decoded - near_max as f64).abs() / near_max as f64;
            assert!(
                relative_error < 1e-10,
                "byte_length={byte_length}: near_max={near_max} decoded={decoded}"
            );
        }
    }

    #[test]
    fn exact_max_raw_at_byte_length_7_is_rejected_not_silently_corrupted() {
        // max_raw() for byte_length=7 is 2^56-1, which is NOT exactly
        // representable as f64 -- `max_raw as f64` rounds UP to 2^56,
        // one past the field's actual capacity. Before the
        // integer-domain bounds check in encode(), this slipped through
        // and write_raw's u8 casts silently truncated every byte to 0
        // instead of erroring. Pin that down.
        let f = field(0, 7, ByteOrder::Big);
        let mut frame = [0xAAu8; 8]; // sentinel, not all-zero
        let before = frame;
        let result = f.encode(f.max_raw() as f64, &mut frame);
        assert!(
            matches!(result, Err(CodecError::ValueOutOfRange { .. })),
            "expected rejection, got {result:?}"
        );
        assert_eq!(frame, before, "a rejected encode must not write anything");
    }

    #[test]
    fn exact_max_raw_at_byte_length_8_round_trips_via_u64_saturation() {
        // Unlike byte_length=7, max_raw() here IS u64::MAX, which is a
        // special case: `u64::MAX as f64` rounds up, but casting that
        // f64 back `as u64` saturates to exactly u64::MAX again (Rust's
        // float->int cast saturates on overflow). The two roundings
        // cancel out at this one specific boundary, so this is the one
        // byte_length=8 value that's safe to round-trip exactly.
        let f = field(0, 8, ByteOrder::Big);
        let mut frame = [0u8; 8];
        f.encode(f.max_raw() as f64, &mut frame).unwrap();
        assert_eq!(f.decode(&frame).unwrap() as u64, u64::MAX);
    }

    #[test]
    fn two_fields_in_one_frame_do_not_cross_talk() {
        let position = field(0, 2, ByteOrder::Big);
        let velocity = field(2, 2, ByteOrder::Little);
        let mut frame = [0u8; 8];
        position.encode(0xABCD_u32 as f64, &mut frame).unwrap();
        velocity.encode(0x1234_u32 as f64, &mut frame).unwrap();
        assert_eq!(position.decode(&frame).unwrap() as u64, 0xABCD);
        assert_eq!(velocity.decode(&frame).unwrap() as u64, 0x1234);
    }

    #[test]
    fn field_overflowing_the_frame_is_rejected_not_truncated() {
        let f = field(6, 4, ByteOrder::Big);
        let frame = [0u8; 8];
        assert_eq!(
            f.decode(&frame),
            Err(CodecError::FieldOutOfFrame {
                byte_offset: 6,
                byte_length: 4
            })
        );
    }

    #[test]
    fn value_above_byte_lengths_range_is_rejected_not_clamped() {
        let f = field(0, 1, ByteOrder::Big); // max_raw = 255
        let mut frame = [0u8; 8];
        let result = f.encode(256.0, &mut frame);
        assert!(matches!(result, Err(CodecError::ValueOutOfRange { .. })));
        assert_eq!(frame, [0u8; 8], "a rejected encode must not write anything");
    }

    #[test]
    fn value_below_zero_raw_is_rejected() {
        let f = FieldCodec {
            byte_offset: 0,
            byte_length: 2,
            byte_order: ByteOrder::Big,
            scale: 1.0,
            offset: 100.0, // raw=0 -> value=100.0, so anything below 100.0 is below raw=0
        };
        let mut frame = [0u8; 8];
        let result = f.encode(50.0, &mut frame);
        assert!(matches!(result, Err(CodecError::ValueOutOfRange { .. })));
    }
}
