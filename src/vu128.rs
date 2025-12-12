//! Unsigned 128-bit VLQ encoding.

use core::fmt::{Debug, Display};

pub(crate) const VU128_BUF_SIZE: usize = 18;

/// Determine encoded length for u128.
///
/// For lengths 1-8: uses standard prefix scheme (same as u64).
/// For length 9: requires encoded second byte >= 0x80.
/// For lengths 10-17: uses extended prefix (first byte 0x00, second byte < 0x80).
/// For length 18: full 128-bit values that need the entire range.
#[inline(always)]
const fn encode_len_vu128(n: u128) -> u8 {
    if n < offset!(2) as u128 {
        return 1;
    }
    if n < offset!(3) as u128 {
        return 2;
    }
    if n < offset!(4) as u128 {
        return 3;
    }
    if n < offset!(5) as u128 {
        return 4;
    }
    if n < offset!(6) as u128 {
        return 5;
    }
    if n < offset!(7) as u128 {
        return 6;
    }
    if n < offset!(8) as u128 {
        return 7;
    }
    if n < offset!(9) as u128 {
        return 8;
    }

    // For 9-byte: value must encode with second byte >= 0x80
    // This means encoded value (n - offset!(9)) >= 2^63
    let nine_byte_min = offset!(9) as u128 + (1u128 << 63);
    let nine_byte_max = offset!(9) as u128 + ((1u128 << 64) - 1);

    if n >= nine_byte_min && n <= nine_byte_max {
        return 9;
    }

    // Extended encoding (10-17 bytes)
    if n < offset!(11) {
        return 10;
    }
    if n < offset!(12) {
        return 11;
    }
    if n < offset!(13) {
        return 12;
    }
    if n < offset!(14) {
        return 13;
    }
    if n < offset!(15) {
        return 14;
    }
    if n < offset!(16) {
        return 15;
    }
    if n < offset!(17) {
        return 16;
    }
    // For values >= offset!(17), use 18-byte raw encoding
    // This simplifies disambiguation (no need to distinguish 17 vs 18)
    18
}

/// Decode length from first two bytes for u128.
#[inline(always)]
pub(crate) const fn decode_len_vu128(first: u8, second: u8) -> u8 {
    let base_len = first.leading_zeros() as u8 + 1;
    if base_len < 9 {
        base_len
    } else {
        // First byte is 0x00
        if second >= 0x80 {
            9 // Standard 9-byte (like Vu64)
        } else if second == 0 {
            18 // Raw 128-bit encoding
        } else {
            // second byte 0x01-0x7F: leading zeros determine length
            // 0x40-0x7F (1 leading zero) -> 10
            // 0x20-0x3F (2 leading zeros) -> 11
            // ...
            // 0x01 (7 leading zeros) -> 16
            9 + second.leading_zeros() as u8
        }
    }
}

/// Encode a u128 in value-length quantity encoding.
#[inline(always)]
#[must_use]
pub const fn encode_vu128(n: u128) -> Vu128 {
    let len = encode_len_vu128(n);
    let mut out_buf = [0u8; VU128_BUF_SIZE];

    match len {
        1 => {
            out_buf[0] = prefix!(1, n as u8);
        }
        2 => {
            let val = (n - offset!(2) as u128) as u16;
            let buf = val.to_be_bytes();
            out_buf[0] = prefix!(2, buf[0]);
            out_buf[1] = buf[1];
        }
        3 => {
            let val = ((n - offset!(3) as u128) as u32) << 8;
            let buf = val.to_be_bytes();
            out_buf[0] = prefix!(3, buf[0]);
            out_buf[1] = buf[1];
            out_buf[2] = buf[2];
        }
        4 => {
            let val = (n - offset!(4) as u128) as u32;
            let buf = val.to_be_bytes();
            out_buf[0] = prefix!(4, buf[0]);
            out_buf[1] = buf[1];
            out_buf[2] = buf[2];
            out_buf[3] = buf[3];
        }
        5 => {
            let val = ((n - offset!(5) as u128) as u64) << (8 * 3);
            let buf = val.to_be_bytes();
            out_buf[0] = prefix!(5, buf[0]);
            out_buf[1] = buf[1];
            out_buf[2] = buf[2];
            out_buf[3] = buf[3];
            out_buf[4] = buf[4];
        }
        6 => {
            let val = ((n - offset!(6) as u128) as u64) << (8 * 2);
            let buf = val.to_be_bytes();
            out_buf[0] = prefix!(6, buf[0]);
            out_buf[1] = buf[1];
            out_buf[2] = buf[2];
            out_buf[3] = buf[3];
            out_buf[4] = buf[4];
            out_buf[5] = buf[5];
        }
        7 => {
            let val = ((n - offset!(7) as u128) as u64) << 8;
            let buf = val.to_be_bytes();
            out_buf[0] = prefix!(7, buf[0]);
            out_buf[1] = buf[1];
            out_buf[2] = buf[2];
            out_buf[3] = buf[3];
            out_buf[4] = buf[4];
            out_buf[5] = buf[5];
            out_buf[6] = buf[6];
        }
        8 => {
            let val = (n - offset!(8) as u128) as u64;
            let buf = val.to_be_bytes();
            out_buf[0] = prefix!(8, buf[0]);
            out_buf[1] = buf[1];
            out_buf[2] = buf[2];
            out_buf[3] = buf[3];
            out_buf[4] = buf[4];
            out_buf[5] = buf[5];
            out_buf[6] = buf[6];
            out_buf[7] = buf[7];
        }
        9 => {
            // Standard 9-byte: [0x00, d1..d8] where d1 >= 0x80
            let val = (n - offset!(9) as u128) as u64;
            let buf = val.to_be_bytes();
            out_buf[0] = 0x00;
            out_buf[1] = buf[0];
            out_buf[2] = buf[1];
            out_buf[3] = buf[2];
            out_buf[4] = buf[3];
            out_buf[5] = buf[4];
            out_buf[6] = buf[5];
            out_buf[7] = buf[6];
            out_buf[8] = buf[7];
        }
        10 => {
            // Extended: [0x00, 01xxxxxx, d2..d9] - 6 bits + 8*8 = 70 bits
            let val = n - offset!(10);
            let buf = val.to_be_bytes();
            out_buf[0] = 0x00;
            // Top 6 bits go in byte 1 with prefix, lower 64 bits in bytes 2-9
            out_buf[1] = prefix!(2, buf[7]); // buf[7] has bits 64-71, we take low 6
            let mut i = 2;
            while i < 10 {
                out_buf[i] = buf[i + 6]; // buf[8..16] -> out_buf[2..10]
                i += 1;
            }
        }
        11 => {
            // Extended: [0x00, 001xxxxx, d2..d10] - 5 bits + 9*8 = 77 bits
            let val = n - offset!(11);
            let buf = val.to_be_bytes();
            out_buf[0] = 0x00;
            out_buf[1] = prefix!(3, buf[6]);
            let mut i = 2;
            while i < 11 {
                out_buf[i] = buf[i + 5]; // buf[7..16] -> out_buf[2..11]
                i += 1;
            }
        }
        12 => {
            // Extended: [0x00, 0001xxxx, d2..d11] - 4 bits + 10*8 = 84 bits
            let val = n - offset!(12);
            let buf = val.to_be_bytes();
            out_buf[0] = 0x00;
            out_buf[1] = prefix!(4, buf[5]);
            let mut i = 2;
            while i < 12 {
                out_buf[i] = buf[i + 4]; // buf[6..16] -> out_buf[2..12]
                i += 1;
            }
        }
        13 => {
            // Extended: [0x00, 00001xxx, d2..d12] - 3 bits + 11*8 = 91 bits
            let val = n - offset!(13);
            let buf = val.to_be_bytes();
            out_buf[0] = 0x00;
            out_buf[1] = prefix!(5, buf[4]);
            let mut i = 2;
            while i < 13 {
                out_buf[i] = buf[i + 3]; // buf[5..16] -> out_buf[2..13]
                i += 1;
            }
        }
        14 => {
            // Extended: [0x00, 000001xx, d2..d13] - 2 bits + 12*8 = 98 bits
            let val = n - offset!(14);
            let buf = val.to_be_bytes();
            out_buf[0] = 0x00;
            out_buf[1] = prefix!(6, buf[3]);
            let mut i = 2;
            while i < 14 {
                out_buf[i] = buf[i + 2]; // buf[4..16] -> out_buf[2..14]
                i += 1;
            }
        }
        15 => {
            // Extended: [0x00, 0000001x, d2..d14] - 1 bit + 13*8 = 105 bits
            let val = n - offset!(15);
            let buf = val.to_be_bytes();
            out_buf[0] = 0x00;
            out_buf[1] = prefix!(7, buf[2]);
            let mut i = 2;
            while i < 15 {
                out_buf[i] = buf[i + 1]; // buf[3..16] -> out_buf[2..15]
                i += 1;
            }
        }
        16 => {
            // Extended: [0x00, 00000001, d2..d15] - 0 bits + 14*8 = 112 bits
            let val = n - offset!(16);
            let buf = val.to_be_bytes();
            out_buf[0] = 0x00;
            out_buf[1] = prefix!(8, buf[1]);
            let mut i = 2;
            while i < 16 {
                out_buf[i] = buf[i]; // buf[2..16] -> out_buf[2..16]
                i += 1;
            }
        }
        _ => {
            // 18 bytes: [0x00, 0x00, d2..d17] - raw 128-bit encoding
            let buf = n.to_be_bytes();
            out_buf[0] = 0x00;
            out_buf[1] = 0x00;
            let mut i = 2;
            while i < 18 {
                out_buf[i] = buf[i - 2]; // buf[0..16] -> out_buf[2..18]
                i += 1;
            }
        }
    };

    Vu128(out_buf)
}

/// Decode a Vu128 back to a native u128.
#[inline(always)]
pub const fn decode_vu128(n: Vu128) -> u128 {
    let len = n.len();
    let b = n.bytes();

    match len {
        1 => unprefix!(1, b[0] as u128),
        2 => {
            u128::from_le_bytes([
                b[1],
                unprefix!(2, b[0]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(2) as u128
        }
        3 => {
            u128::from_le_bytes([
                b[2],
                b[1],
                unprefix!(3, b[0]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(3) as u128
        }
        4 => {
            u128::from_le_bytes([
                b[3],
                b[2],
                b[1],
                unprefix!(4, b[0]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(4) as u128
        }
        5 => {
            u128::from_le_bytes([
                b[4],
                b[3],
                b[2],
                b[1],
                unprefix!(5, b[0]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(5) as u128
        }
        6 => {
            u128::from_le_bytes([
                b[5],
                b[4],
                b[3],
                b[2],
                b[1],
                unprefix!(6, b[0]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(6) as u128
        }
        7 => {
            u128::from_le_bytes([
                b[6],
                b[5],
                b[4],
                b[3],
                b[2],
                b[1],
                unprefix!(7, b[0]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(7) as u128
        }
        8 => {
            u128::from_le_bytes([
                b[7],
                b[6],
                b[5],
                b[4],
                b[3],
                b[2],
                b[1],
                unprefix!(8, b[0]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(8) as u128
        }
        9 => {
            // [0x00, d1..d8] - 64 bits of data
            u128::from_le_bytes([
                b[8], b[7], b[6], b[5], b[4], b[3], b[2], b[1], 0, 0, 0, 0, 0, 0, 0, 0,
            ]) + offset!(9) as u128
        }
        10 => {
            // [0x00, 01xxxxxx, d2..d9] - 6 bits in b[1], 64 bits in b[2..10]
            u128::from_le_bytes([
                b[9],
                b[8],
                b[7],
                b[6],
                b[5],
                b[4],
                b[3],
                b[2],
                unprefix!(2, b[1]),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(10)
        }
        11 => {
            // [0x00, 001xxxxx, d2..d10] - 5 bits + 72 bits
            u128::from_le_bytes([
                b[10],
                b[9],
                b[8],
                b[7],
                b[6],
                b[5],
                b[4],
                b[3],
                b[2],
                unprefix!(3, b[1]),
                0,
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(11)
        }
        12 => {
            // [0x00, 0001xxxx, d2..d11] - 4 bits + 80 bits
            u128::from_le_bytes([
                b[11],
                b[10],
                b[9],
                b[8],
                b[7],
                b[6],
                b[5],
                b[4],
                b[3],
                b[2],
                unprefix!(4, b[1]),
                0,
                0,
                0,
                0,
                0,
            ]) + offset!(12)
        }
        13 => {
            // [0x00, 00001xxx, d2..d12] - 3 bits + 88 bits
            u128::from_le_bytes([
                b[12],
                b[11],
                b[10],
                b[9],
                b[8],
                b[7],
                b[6],
                b[5],
                b[4],
                b[3],
                b[2],
                unprefix!(5, b[1]),
                0,
                0,
                0,
                0,
            ]) + offset!(13)
        }
        14 => {
            // [0x00, 000001xx, d2..d13] - 2 bits + 96 bits
            u128::from_le_bytes([
                b[13],
                b[12],
                b[11],
                b[10],
                b[9],
                b[8],
                b[7],
                b[6],
                b[5],
                b[4],
                b[3],
                b[2],
                unprefix!(6, b[1]),
                0,
                0,
                0,
            ]) + offset!(14)
        }
        15 => {
            // [0x00, 0000001x, d2..d14] - 1 bit + 104 bits
            u128::from_le_bytes([
                b[14],
                b[13],
                b[12],
                b[11],
                b[10],
                b[9],
                b[8],
                b[7],
                b[6],
                b[5],
                b[4],
                b[3],
                b[2],
                unprefix!(7, b[1]),
                0,
                0,
            ]) + offset!(15)
        }
        16 => {
            // [0x00, 00000001, d2..d15] - 0 bits + 112 bits
            u128::from_le_bytes([
                b[15], b[14], b[13], b[12], b[11], b[10], b[9], b[8], b[7], b[6], b[5], b[4], b[3],
                b[2], 0, // byte 1 has no data bits
                0,
            ]) + offset!(16)
        }
        _ => {
            // 18 bytes: [0x00, 0x00, d2..d17] - raw 128-bit encoding
            u128::from_le_bytes([
                b[17], b[16], b[15], b[14], b[13], b[12], b[11], b[10], b[9], b[8], b[7], b[6],
                b[5], b[4], b[3], b[2],
            ])
        }
    }
}

/// An unsigned 128-bit integer in value-length quantity encoding.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vu128(pub(crate) [u8; VU128_BUF_SIZE]);

#[allow(clippy::len_without_is_empty)]
impl Vu128 {
    /// Construct a new VLQ instance from the given `u128`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u128) -> Vu128 {
        encode_vu128(value)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu128(self.0[0], self.0[1])
    }

    /// Retrieve the stored number as `u128`.
    #[inline(always)]
    pub const fn get(&self) -> u128 {
        decode_vu128(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 18] {
        self.0
    }

    /// Get the serialized representation of the VLQ as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.0[..(self.len() as usize)]
    }
}

impl From<u128> for Vu128 {
    fn from(n: u128) -> Self {
        encode_vu128(n)
    }
}

impl From<Vu128> for u128 {
    fn from(n: Vu128) -> Self {
        decode_vu128(n)
    }
}

impl Display for Vu128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl Debug for Vu128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vu128(0b")?;
        for x in self.0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0[len]))
    }
}
