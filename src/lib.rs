//! ## Algorithm
//!
//! Fast VLQ is a variant of value-length quantity encoding, with a focus on encoding and
//! decoding speed. The total number of bytes can always be derived from the very first byte,
//! and unlike VLQ, the integer type supported is `u64` exclusively, and will
//! take up a maximum of 9 bytes (for values greater than 56-bit).
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
//! fastvlq = "1"
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::cast_lossless)]

extern crate core;

macro_rules! prefix {
    (1) => {
        0b1000_0000
    };
    (2) => {
        0b0100_0000
    };
    (3) => {
        0b0010_0000
    };
    (4) => {
        0b0001_0000
    };
    (5) => {
        0b0000_1000
    };
    (6) => {
        0b0000_0100
    };
    (7) => {
        0b0000_0010
    };
    (8) => {
        0b0000_0001
    };
    (9) => {
        0b0000_0000
    };
    (1, $target:expr) => {
        $target | 0b1000_0000
    };
    (2, $target:expr) => {
        (0b0011_1111 & $target) | 0b0100_0000
    };
    (3, $target:expr) => {
        (0b0001_1111 & $target) | 0b0010_0000
    };
    (4, $target:expr) => {
        (0b0000_1111 & $target) | 0b0001_0000
    };
    (5, $target:expr) => {
        (0b0000_0111 & $target) | 0b0000_1000
    };
    (6, $target:expr) => {
        (0b0000_0011 & $target) | 0b0000_0100
    };
    (7, $target:expr) => {
        (0b0000_0001 & $target) | 0b0000_0010
    };
    (8, $target:expr) => {
        0b0000_0001
    };
    (9, $target:expr) => {
        0b0000_0000
    };
}

macro_rules! unprefix {
    (1, $target:expr) => {
        $target & 0b0111_1111
    };
    (2, $target:expr) => {
        $target & 0b0011_1111
    };
    (3, $target:expr) => {
        $target & 0b0001_1111
    };
    (4, $target:expr) => {
        $target & 0b0000_1111
    };
    (5, $target:expr) => {
        $target & 0b0000_0111
    };
    (6, $target:expr) => {
        $target & 0b0000_0011
    };
    (7, $target:expr) => {
        $target & 0b0000_0001
    };
    (8, $target:expr) => {
        0b0000_0000
    };
    (9, $target:expr) => {
        0b0000_0000
    };
}

macro_rules! offset {
    (1) => {
        0
    };
    (2) => {
        1 << 7
    };
    (3) => {
        offset!(2) as u32 + (1 << 14)
    };
    (4) => {
        offset!(3) as u32 + (1 << 21)
    };
    (5) => {
        offset!(4) as u64 + (1 << 28)
    };
    (6) => {
        offset!(5) + (1 << 35)
    };
    (7) => {
        offset!(6) + (1 << 42)
    };
    (8) => {
        offset!(7) + (1 << 49)
    };
    (9) => {
        offset!(8) + (1 << 56)
    };
}

macro_rules! encode_offset {
    (2, $n:tt) => {
        $n as u16 - offset!(2)
    };
    (3, $n:tt) => {
        ($n as u32 - offset!(3)) << 8
    };
    (4, $n:tt) => {
        ($n as u32 - offset!(4))
    };
    (5, $n:tt) => {
        ($n as u64 - offset!(5)) << (8 * 3)
    };
    (6, $n:tt) => {
        ($n as u64 - offset!(6)) << (8 * 2)
    };
    (7, $n:tt) => {
        ($n as u64 - offset!(7)) << 8
    };
    (8, $n:tt) => {
        ($n as u64 - offset!(8))
    };
    (9, $n:tt) => {
        ($n as u64 - offset!(9))
    };
}

/// Decoding bit depth by prefix in bits:
///
/// 1xxx_xxxx: 1 byte
/// 01xx_xxxx: 2 bytes
/// 001x_xxxx: 3 bytes
/// 0001_xxxx: 4 bytes
/// 0000_1xxx: 5 bytes
/// 0000_01xx: 6 bytes
/// 0000_001x: 7 bytes
/// 0000_0001: 8 bytes
/// 0000_0000: 9 bytes
#[inline(always)]
const fn decode_len_vu64(n: u8) -> u8 {
    n.leading_zeros() as u8 + 1
}

/// Encoding bit depth by length in bytes:
///
/// 1: 7 bits
/// 2: 14 (6 + 8) bits
/// 3: 20 (5 + 8 * 2) bits
/// 4: 28 (4 + 8 * 3) bits
/// 5: 35 (3 + 8 * 4) bits
/// 6: 42 (2 + 8 * 5) bits
/// 7: 49 (0 + 8 * 6) bits
/// 8: 56 (1 + 8 * 7) bits
/// 9: 64 (1 + 8 * 8) bits
#[inline(always)]
const fn encode_len_vu64(n: u64) -> u8 {
    match n {
        n if n < offset!(2) as u64 => 1,
        n if n < offset!(3) as u64 => 2,
        n if n < offset!(4) as u64 => 3,
        n if n < offset!(5) => 4,
        n if n < offset!(6) => 5,
        n if n < offset!(7) => 6,
        n if n < offset!(8) => 7,
        n if n < offset!(9) => 8,
        _ => 9,
    }
}

macro_rules! copy_from_slice_offset {
    (source = $source:ident, dest = $dest:ident, offset = $offset:tt) => {
        let mut i = 0;
        while i < $offset {
            $dest[i] = $source[i];
            i += 1;
        }
    };
}

#[inline(always)]
pub const fn encode_vu64(n: u64) -> Vu64 {
    let len = encode_len_vu64(n);
    let mut out_buf = [0u8; VU64_BUF_SIZE];

    match len {
        1 => {
            out_buf[0] = prefix!(1, n as u8);
        }
        2 => {
            let buf = encode_offset!(2, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 2);
            out_buf[0] = prefix!(2, buf[0]);
        }
        3 => {
            let buf = encode_offset!(3, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 3);
            out_buf[0] = prefix!(3, buf[0]);
        }
        4 => {
            let buf = encode_offset!(4, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 4);
            out_buf[0] = prefix!(4, buf[0]);
        }
        5 => {
            let buf = encode_offset!(5, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 5);
            out_buf[0] = prefix!(5, buf[0]);
        }
        6 => {
            let buf = encode_offset!(6, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 6);
            out_buf[0] = prefix!(6, buf[0]);
        }
        7 => {
            let buf = encode_offset!(7, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 7);
            out_buf[0] = prefix!(7, buf[0]);
        }
        8 => {
            let buf = encode_offset!(8, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 8);
            out_buf[0] = prefix!(8, buf[0]);
        }
        _ => {
            let buf = encode_offset!(9, n).to_be_bytes();
            let mut i = 0;
            while i < 8 {
                out_buf[i + 1] = buf[i];
                i += 1;
            }
            out_buf[0] = prefix!(9, buf[0]);
        }
    };

    Vu64(out_buf)
}

#[inline(always)]
pub const fn decode_vu64(n: Vu64) -> u64 {
    let len = n.len();
    let n = n.bytes();

    match len {
        1 => unprefix!(1, n[0] as u64),
        2 => u64::from_le_bytes([n[1], unprefix!(2, n[0]), 0, 0, 0, 0, 0, 0]) + offset!(2) as u64,
        3 => {
            u64::from_le_bytes([n[2], n[1], unprefix!(3, n[0]), 0, 0, 0, 0, 0]) + offset!(3) as u64
        }
        4 => {
            u64::from_le_bytes([n[3], n[2], n[1], unprefix!(4, n[0]), 0, 0, 0, 0])
                + offset!(4) as u64
        }
        5 => {
            u64::from_le_bytes([n[4], n[3], n[2], n[1], unprefix!(5, n[0]), 0, 0, 0])
                + offset!(5) as u64
        }
        6 => {
            u64::from_le_bytes([n[5], n[4], n[3], n[2], n[1], unprefix!(6, n[0]), 0, 0])
                + offset!(6) as u64
        }
        7 => {
            u64::from_le_bytes([n[6], n[5], n[4], n[3], n[2], n[1], unprefix!(7, n[0]), 0])
                + offset!(7) as u64
        }
        8 => {
            u64::from_le_bytes([n[7], n[6], n[5], n[4], n[3], n[2], n[1], unprefix!(8, n[0])])
                + offset!(8) as u64
        }
        _ => {
            u64::from_le_bytes([n[8], n[7], n[6], n[5], n[4], n[3], n[2], n[1]]) + offset!(9) as u64
        }
    }
}

const VU64_BUF_SIZE: usize = 9;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vu64([u8; VU64_BUF_SIZE]);

#[allow(clippy::len_without_is_empty)]
impl Vu64 {
    #[inline(always)]
    pub const fn new(value: u64) -> Vu64 {
        encode_vu64(value)
    }

    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu64(self.0[0])
    }

    #[inline(always)]
    pub const fn get(&self) -> u64 {
        decode_vu64(*self)
    }

    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 9] {
        self.0
    }
}

impl From<u64> for Vu64 {
    fn from(n: u64) -> Self {
        encode_vu64(n)
    }
}

impl From<Vu64> for u64 {
    fn from(n: Vu64) -> Self {
        decode_vu64(n)
    }
}

impl core::fmt::Display for Vu64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.get(), f)
    }
}

impl core::fmt::Debug for Vu64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vu64(0b")?;
        for x in self.0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0[len]))
    }
}

#[cfg(feature = "std")]
pub trait ReadVu64Ext<T> {
    /// Read a variable-length `u64`.
    fn read_vu64(&mut self) -> std::io::Result<T>;
}

#[cfg(feature = "std")]
pub trait WriteVu64Ext<T> {
    /// Write a variable-length `u64`.
    fn write_vu64(&mut self, n: T) -> std::io::Result<()>;
}

#[cfg(feature = "std")]
impl<R: std::io::Read> ReadVu64Ext<u64> for R {
    fn read_vu64(&mut self) -> std::io::Result<u64> {
        let mut buf: [u8; 9] = [0; 9];
        self.read_exact(&mut buf[0..1])?;

        let len = decode_len_vu64(buf[0]);
        if len != 1 {
            self.read_exact(&mut buf[1..len as usize])?;
        }

        let vlq = Vu64(buf);
        Ok(decode_vu64(vlq))
    }
}

#[cfg(feature = "std")]
impl<W: std::io::Write> WriteVu64Ext<u64> for W {
    fn write_vu64(&mut self, n: u64) -> std::io::Result<()> {
        let vlq = encode_vu64(n);
        self.write_all(&vlq.0[0..vlq.len() as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(decode_vu64(encode_vu64(0x7F)), 0x7F, "max for 1");
        assert_eq!(decode_vu64(encode_vu64(0x80)), 0x80, "min for 2");
        assert_eq!(decode_vu64(encode_vu64(0x3FFF)), 0x3FFF, "max for 2");
        assert_eq!(decode_vu64(encode_vu64(0x4000)), 0x4000, "min for 3");
        assert_eq!(decode_vu64(encode_vu64(0x0F_FFFF)), 0x0F_FFFF, "max for 3");
        assert_eq!(decode_vu64(encode_vu64(0x20_0000)), 0x20_0000, "min for 4");
        assert_eq!(
            decode_vu64(encode_vu64(0x1FFF_FFFF)),
            0x1FFF_FFFF,
            "max for 4"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x2000_0000)),
            0x2000_0000,
            "min for 5"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x17_FFFF_FFFF)),
            0x17_FFFF_FFFF,
            "max for 5"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x18_0000_0000)),
            0x18_0000_0000,
            "min for 6"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x13FF_FFFF_FFFF)),
            0x13FF_FFFF_FFFF,
            "max for 6"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x1411_1111_1111)),
            0x1411_1111_1111,
            "min for 7"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x10_FFFF_FFFF_FFFF)),
            0x10_FFFF_FFFF_FFFF,
            "max for 7"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x12_1111_1111_1111)),
            0x12_1111_1111_1111,
            "min for 8"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x11FF_FFFF_FFFF_FFFF)),
            0x11FF_FFFF_FFFF_FFFF,
            "max for 8"
        );
        assert_eq!(
            decode_vu64(encode_vu64(0x1011_1111_1111_1111)),
            0x1011_1111_1111_1111,
            "min for 9"
        );
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
}
