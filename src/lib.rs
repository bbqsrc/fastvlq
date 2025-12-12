//! ## Algorithm
//!
//! Fast VLQ is a variant of value-length quantity encoding, with a focus on encoding and
//! decoding speed. The total number of bytes can always be derived from the very first byte.
//!
//! Supported types:
//! - `Vu32` / `Vi32`: unsigned/signed 32-bit (max 5 bytes)
//! - `Vu64` / `Vi64`: unsigned/signed 64-bit (max 9 bytes)
//! - `Vu128` / `Vi128`: unsigned/signed 128-bit (max 18 bytes)
//!
//! Signed types use zigzag encoding for efficient storage of small absolute values.
//!
//! This crate does not enforce an invariant that a number may only have one representation,
//! which means that it is possible to encode `1` as, for example, both `0b1000_0001` and
//! `0b0100_0000_0000_0001`.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! fastvlq = "2"
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::cast_lossless)]
#![deny(missing_docs)]

#[cfg(feature = "async-futures")]
mod futures;
#[cfg(feature = "async-tokio")]
mod tokio;

#[macro_use]
mod macros;

mod vi128;
mod vi32;
mod vi64;
mod vu128;
mod vu32;
mod vu64;

#[cfg(feature = "std")]
use std::io::{Read, Result as IoResult, Write};

pub use vi32::{Vi32, decode_vi32, encode_vi32};
pub use vi64::{Vi64, decode_vi64, encode_vi64};
pub use vi128::{Vi128, decode_vi128, encode_vi128};
pub use vu32::{Vu32, decode_vu32, encode_vu32};
pub use vu64::{Vu64, decode_vu64, encode_vu64};
pub use vu128::{Vu128, decode_vu128, encode_vu128};

#[cfg(feature = "async-futures")]
pub use futures::{FuturesReadVlqExt, FuturesWriteVlqExt};
#[cfg(feature = "async-tokio")]
pub use tokio::{TokioReadVlqExt, TokioWriteVlqExt};

#[cfg(feature = "std")]
/// Extension trait for reading VLQ-encoded integers from a reader.
pub trait ReadVlqExt {
    /// Read a variable-length `u32`.
    fn read_vu32(&mut self) -> IoResult<u32>;
    /// Read a variable-length `i32`.
    fn read_vi32(&mut self) -> IoResult<i32>;
    /// Read a variable-length `u64`.
    fn read_vu64(&mut self) -> IoResult<u64>;
    /// Read a variable-length `i64`.
    fn read_vi64(&mut self) -> IoResult<i64>;
    /// Read a variable-length `u128`.
    fn read_vu128(&mut self) -> IoResult<u128>;
    /// Read a variable-length `i128`.
    fn read_vi128(&mut self) -> IoResult<i128>;
}

#[cfg(feature = "std")]
/// Extension trait for writing VLQ-encoded integers to a writer.
pub trait WriteVlqExt {
    /// Write a variable-length `u32`.
    fn write_vu32(&mut self, n: u32) -> IoResult<()>;
    /// Write a variable-length `i32`.
    fn write_vi32(&mut self, n: i32) -> IoResult<()>;
    /// Write a variable-length `u64`.
    fn write_vu64(&mut self, n: u64) -> IoResult<()>;
    /// Write a variable-length `i64`.
    fn write_vi64(&mut self, n: i64) -> IoResult<()>;
    /// Write a variable-length `u128`.
    fn write_vu128(&mut self, n: u128) -> IoResult<()>;
    /// Write a variable-length `i128`.
    fn write_vi128(&mut self, n: i128) -> IoResult<()>;
}

#[cfg(feature = "std")]
impl<R: Read> ReadVlqExt for R {
    fn read_vu32(&mut self) -> IoResult<u32> {
        let mut buf = [0u8; vu32::VU32_BUF_SIZE];
        self.read_exact(&mut buf[0..1])?;
        let len = vu32::decode_len_vu32(buf[0]) as usize;
        if len > 1 {
            self.read_exact(&mut buf[1..len])?;
        }
        Ok(decode_vu32(vu32::Vu32(buf)))
    }

    fn read_vi32(&mut self) -> IoResult<i32> {
        self.read_vu32().map(vi32::zigzag_decode_i32)
    }

    fn read_vu64(&mut self) -> IoResult<u64> {
        let mut buf = [0u8; vu64::VU64_BUF_SIZE];
        self.read_exact(&mut buf[0..1])?;
        let len = vu64::decode_len_vu64(buf[0]) as usize;
        if len > 1 {
            self.read_exact(&mut buf[1..len])?;
        }
        Ok(decode_vu64(vu64::Vu64(buf)))
    }

    fn read_vi64(&mut self) -> IoResult<i64> {
        self.read_vu64().map(vi64::zigzag_decode_i64)
    }

    fn read_vu128(&mut self) -> IoResult<u128> {
        let mut buf = [0u8; vu128::VU128_BUF_SIZE];
        self.read_exact(&mut buf[0..1])?;
        // Need second byte to determine extended length
        if buf[0] == 0 {
            self.read_exact(&mut buf[1..2])?;
        }
        let len = vu128::decode_len_vu128(buf[0], buf[1]) as usize;
        if len > 2 {
            self.read_exact(&mut buf[2..len])?;
        } else if len == 2 && buf[0] != 0 {
            // Standard 2-byte (not extended), already read first byte
            self.read_exact(&mut buf[1..2])?;
        }
        Ok(decode_vu128(vu128::Vu128(buf)))
    }

    fn read_vi128(&mut self) -> IoResult<i128> {
        self.read_vu128().map(vi128::zigzag_decode_i128)
    }
}

#[cfg(feature = "std")]
impl<W: Write> WriteVlqExt for W {
    fn write_vu32(&mut self, n: u32) -> IoResult<()> {
        self.write_all(encode_vu32(n).as_slice())
    }

    fn write_vi32(&mut self, n: i32) -> IoResult<()> {
        self.write_vu32(vi32::zigzag_encode_i32(n))
    }

    fn write_vu64(&mut self, n: u64) -> IoResult<()> {
        self.write_all(encode_vu64(n).as_slice())
    }

    fn write_vi64(&mut self, n: i64) -> IoResult<()> {
        self.write_vu64(vi64::zigzag_encode_i64(n))
    }

    fn write_vu128(&mut self, n: u128) -> IoResult<()> {
        self.write_all(encode_vu128(n).as_slice())
    }

    fn write_vi128(&mut self, n: i128) -> IoResult<()> {
        self.write_vu128(vi128::zigzag_encode_i128(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vi32::zigzag_encode_i32;
    use vi64::zigzag_encode_i64;
    use vi128::zigzag_encode_i128;
    use vu64::decode_len_vu64;

    #[test]
    fn check_decode() {
        assert_eq!(decode_len_vu64(0b1111_1111), 1, "max for 1");
        assert_eq!(decode_len_vu64(0b1000_0000), 1, "min for 1");
        assert_eq!(decode_len_vu64(0b0111_1111), 2, "max for 2");
        assert_eq!(decode_len_vu64(0b0100_0000), 2, "min for 2");
        assert_eq!(decode_len_vu64(0b0011_1111), 3, "max for 3");
        assert_eq!(decode_len_vu64(0b0010_0000), 3, "min for 3");
        assert_eq!(decode_len_vu64(0b0001_1111), 4, "max for 4");
        assert_eq!(decode_len_vu64(0b0001_0000), 4, "min for 4");
        assert_eq!(decode_len_vu64(0b0000_1111), 5, "max for 5");
        assert_eq!(decode_len_vu64(0b0000_1000), 5, "min for 5");
        assert_eq!(decode_len_vu64(0b0000_0111), 6, "max for 6");
        assert_eq!(decode_len_vu64(0b0000_0100), 6, "min for 6");
        assert_eq!(decode_len_vu64(0b0000_0011), 7, "max for 7");
        assert_eq!(decode_len_vu64(0b0000_0010), 7, "min for 7");
        assert_eq!(decode_len_vu64(0b0000_0001), 8, "min/max for 8");
        assert_eq!(decode_len_vu64(0b0000_0000), 9, "min/max for 9");
    }

    #[test]
    fn round_trip() {
        assert_eq!(decode_vu64(encode_vu64(core::u64::MIN)), core::u64::MIN);
        // 1-byte: 0x00 to 0x7F
        assert_eq!(encode_vu64(0x7F).len(), 1);
        assert_eq!(decode_vu64(encode_vu64(0x7F)), 0x7F, "max for 1");
        // 2-byte: 0x80 to 0x407F
        assert_eq!(encode_vu64(0x80).len(), 2);
        assert_eq!(decode_vu64(encode_vu64(0x80)), 0x80, "min for 2");
        assert_eq!(encode_vu64(0x407F).len(), 2);
        assert_eq!(decode_vu64(encode_vu64(0x407F)), 0x407F, "max for 2");
        // 3-byte: 0x4080 to 0x20_407F
        assert_eq!(encode_vu64(0x4080).len(), 3);
        assert_eq!(decode_vu64(encode_vu64(0x4080)), 0x4080, "min for 3");
        assert_eq!(encode_vu64(0x20_407F).len(), 3);
        assert_eq!(decode_vu64(encode_vu64(0x20_407F)), 0x20_407F, "max for 3");
        // 4-byte: 0x20_4080 to 0x1020_407F
        assert_eq!(encode_vu64(0x20_4080).len(), 4);
        assert_eq!(decode_vu64(encode_vu64(0x20_4080)), 0x20_4080, "min for 4");
        assert_eq!(encode_vu64(0x1020_407F).len(), 4);
        assert_eq!(
            decode_vu64(encode_vu64(0x1020_407F)),
            0x1020_407F,
            "max for 4"
        );
        // 5-byte: 0x1020_4080 to 0x8_1020_407F
        assert_eq!(encode_vu64(0x1020_4080).len(), 5);
        assert_eq!(
            decode_vu64(encode_vu64(0x1020_4080)),
            0x1020_4080,
            "min for 5"
        );
        assert_eq!(encode_vu64(0x8_1020_407F).len(), 5);
        assert_eq!(
            decode_vu64(encode_vu64(0x8_1020_407F)),
            0x8_1020_407F,
            "max for 5"
        );
        // 6-byte: 0x8_1020_4080 to 0x408_1020_407F
        assert_eq!(encode_vu64(0x8_1020_4080).len(), 6);
        assert_eq!(
            decode_vu64(encode_vu64(0x8_1020_4080)),
            0x8_1020_4080,
            "min for 6"
        );
        assert_eq!(encode_vu64(0x408_1020_407F).len(), 6);
        assert_eq!(
            decode_vu64(encode_vu64(0x408_1020_407F)),
            0x408_1020_407F,
            "max for 6"
        );
        // 7-byte: 0x408_1020_4080 to 0x2_0408_1020_407F
        assert_eq!(encode_vu64(0x408_1020_4080).len(), 7);
        assert_eq!(
            decode_vu64(encode_vu64(0x408_1020_4080)),
            0x408_1020_4080,
            "min for 7"
        );
        assert_eq!(encode_vu64(0x2_0408_1020_407F).len(), 7);
        assert_eq!(
            decode_vu64(encode_vu64(0x2_0408_1020_407F)),
            0x2_0408_1020_407F,
            "max for 7"
        );
        // 8-byte: 0x2_0408_1020_4080 to 0x102_0408_1020_407F
        assert_eq!(encode_vu64(0x2_0408_1020_4080).len(), 8);
        assert_eq!(
            decode_vu64(encode_vu64(0x2_0408_1020_4080)),
            0x2_0408_1020_4080,
            "min for 8"
        );
        assert_eq!(encode_vu64(0x102_0408_1020_407F).len(), 8);
        assert_eq!(
            decode_vu64(encode_vu64(0x102_0408_1020_407F)),
            0x102_0408_1020_407F,
            "max for 8"
        );
        // 9-byte: 0x102_0408_1020_4080 to u64::MAX
        assert_eq!(encode_vu64(0x102_0408_1020_4080).len(), 9);
        assert_eq!(
            decode_vu64(encode_vu64(0x102_0408_1020_4080)),
            0x102_0408_1020_4080,
            "min for 9"
        );
        assert_eq!(encode_vu64(core::u64::MAX).len(), 9);
        assert_eq!(decode_vu64(encode_vu64(core::u64::MAX)), core::u64::MAX);
        assert_eq!(
            decode_vu64(encode_vu64(core::i64::MIN as u64)) as i64,
            core::i64::MIN
        );

        assert_eq!(1, decode_vu64(encode_vu64(0x1)), "1");
        assert_eq!(0, decode_vu64(encode_vu64(0x0)), "0");
        assert_eq!(0x011, decode_vu64(encode_vu64(0x011)), "2");
        assert_eq!(0xFF221122, decode_vu64(encode_vu64(0xFF221122)), "3");
        assert_eq!(
            0x11FF_FFFF_FFFF_FFFF,
            decode_vu64(encode_vu64(0x11FF_FFFF_FFFF_FFFF)),
            "4"
        );
        assert_eq!(
            0x1011_1111_1111_1111,
            decode_vu64(encode_vu64(0x1011_1111_1111_1111)),
            "5"
        );
        assert_eq!(
            core::u64::MAX,
            decode_vu64(encode_vu64(core::u64::MAX)),
            "max"
        );
    }

    #[test]
    fn zigzag_i32() {
        assert_eq!(zigzag_encode_i32(0), 0);
        assert_eq!(zigzag_encode_i32(-1), 1);
        assert_eq!(zigzag_encode_i32(1), 2);
        assert_eq!(zigzag_encode_i32(-2), 3);
        assert_eq!(zigzag_encode_i32(2), 4);
        assert_eq!(
            vi32::zigzag_decode_i32(zigzag_encode_i32(i32::MIN)),
            i32::MIN
        );
        assert_eq!(
            vi32::zigzag_decode_i32(zigzag_encode_i32(i32::MAX)),
            i32::MAX
        );
    }

    #[test]
    fn zigzag_i64() {
        assert_eq!(zigzag_encode_i64(0), 0);
        assert_eq!(zigzag_encode_i64(-1), 1);
        assert_eq!(zigzag_encode_i64(1), 2);
        assert_eq!(
            vi64::zigzag_decode_i64(zigzag_encode_i64(i64::MIN)),
            i64::MIN
        );
        assert_eq!(
            vi64::zigzag_decode_i64(zigzag_encode_i64(i64::MAX)),
            i64::MAX
        );
    }

    #[test]
    fn zigzag_i128() {
        assert_eq!(zigzag_encode_i128(0), 0);
        assert_eq!(zigzag_encode_i128(-1), 1);
        assert_eq!(zigzag_encode_i128(1), 2);
        assert_eq!(
            vi128::zigzag_decode_i128(zigzag_encode_i128(i128::MIN)),
            i128::MIN
        );
        assert_eq!(
            vi128::zigzag_decode_i128(zigzag_encode_i128(i128::MAX)),
            i128::MAX
        );
    }

    #[test]
    fn vu32_round_trip() {
        assert_eq!(decode_vu32(encode_vu32(0)), 0);
        assert_eq!(decode_vu32(encode_vu32(127)), 127);
        assert_eq!(decode_vu32(encode_vu32(128)), 128);
        assert_eq!(decode_vu32(encode_vu32(u32::MAX)), u32::MAX);

        // Check lengths
        assert_eq!(encode_vu32(0).len(), 1);
        assert_eq!(encode_vu32(127).len(), 1);
        assert_eq!(encode_vu32(128).len(), 2);
        assert_eq!(encode_vu32(u32::MAX).len(), 5);
    }

    #[test]
    fn vi32_round_trip() {
        assert_eq!(decode_vi32(encode_vi32(0)), 0);
        assert_eq!(decode_vi32(encode_vi32(1)), 1);
        assert_eq!(decode_vi32(encode_vi32(-1)), -1);
        assert_eq!(decode_vi32(encode_vi32(i32::MIN)), i32::MIN);
        assert_eq!(decode_vi32(encode_vi32(i32::MAX)), i32::MAX);

        // Small values should be compact
        assert_eq!(encode_vi32(0).len(), 1);
        assert_eq!(encode_vi32(1).len(), 1);
        assert_eq!(encode_vi32(-1).len(), 1);
    }

    #[test]
    fn vi64_round_trip() {
        assert_eq!(decode_vi64(encode_vi64(0)), 0);
        assert_eq!(decode_vi64(encode_vi64(1)), 1);
        assert_eq!(decode_vi64(encode_vi64(-1)), -1);
        assert_eq!(decode_vi64(encode_vi64(i64::MIN)), i64::MIN);
        assert_eq!(decode_vi64(encode_vi64(i64::MAX)), i64::MAX);

        // Small values should be compact
        assert_eq!(encode_vi64(0).len(), 1);
        assert_eq!(encode_vi64(1).len(), 1);
        assert_eq!(encode_vi64(-1).len(), 1);
    }

    #[test]
    fn vu128_round_trip_small() {
        // Small values (same as u64 range)
        assert_eq!(decode_vu128(encode_vu128(0)), 0);
        assert_eq!(decode_vu128(encode_vu128(127)), 127);
        assert_eq!(decode_vu128(encode_vu128(128)), 128);
        assert_eq!(
            decode_vu128(encode_vu128(u64::MAX as u128)),
            u64::MAX as u128
        );
    }

    #[test]
    fn vu128_round_trip_large() {
        // Large values (beyond u64 range)
        let val = u64::MAX as u128 + 1;
        assert_eq!(decode_vu128(encode_vu128(val)), val);

        assert_eq!(decode_vu128(encode_vu128(u128::MAX)), u128::MAX);
    }

    #[test]
    fn vi128_round_trip() {
        assert_eq!(decode_vi128(encode_vi128(0)), 0);
        assert_eq!(decode_vi128(encode_vi128(1)), 1);
        assert_eq!(decode_vi128(encode_vi128(-1)), -1);
        assert_eq!(decode_vi128(encode_vi128(i128::MIN)), i128::MIN);
        assert_eq!(decode_vi128(encode_vi128(i128::MAX)), i128::MAX);
    }
}

#[cfg(all(feature = "std", test))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn roundtrip_u64(x: u64) {
            prop_assert_eq!(u64::from(Vu64::from(x)), x);
        }

        #[test]
        fn roundtrip_i64(x: i64) {
            prop_assert_eq!(i64::from(Vi64::from(x)), x);
        }

        #[test]
        fn roundtrip_u32(x: u32) {
            prop_assert_eq!(u32::from(Vu32::from(x)), x);
        }

        #[test]
        fn roundtrip_i32(x: i32) {
            prop_assert_eq!(i32::from(Vi32::from(x)), x);
        }

        #[test]
        fn roundtrip_u128(x: u128) {
            prop_assert_eq!(u128::from(Vu128::from(x)), x);
        }

        #[test]
        fn roundtrip_i128(x: i128) {
            prop_assert_eq!(i128::from(Vi128::from(x)), x);
        }
    }
}
