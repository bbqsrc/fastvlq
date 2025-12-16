//! Unsigned 128-bit VLQ encoding.

use core::fmt::{Debug, Display};
use core::marker::PhantomData;

use crate::{BE, LE};

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
            9 + second.leading_zeros() as u8
        }
    }
}

/// Encode a u128 in big-endian VLQ format.
#[inline(always)]
#[must_use]
pub const fn encode_vu128_be(n: u128) -> Vu128<BE> {
    let len = encode_len_vu128(n);

    let out: [u8; 18] = match len {
        1 => [
            0x80 | (n as u8),
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
            0,
            0,
            0,
        ],
        2 => {
            let val = n - offset!(2) as u128;
            let b = val.to_be_bytes();
            [
                0x40 | ((val >> 8) as u8),
                b[15],
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
                0,
                0,
            ]
        }
        3 => {
            let val = n - offset!(3) as u128;
            let b = val.to_be_bytes();
            [
                0x20 | ((val >> 16) as u8),
                b[14],
                b[15],
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
                0,
            ]
        }
        4 => {
            let val = n - offset!(4) as u128;
            let b = val.to_be_bytes();
            [
                0x10 | ((val >> 24) as u8),
                b[13],
                b[14],
                b[15],
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
            ]
        }
        5 => {
            let val = n - offset!(5) as u128;
            let b = val.to_be_bytes();
            [
                0x08 | ((val >> 32) as u8),
                b[12],
                b[13],
                b[14],
                b[15],
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
            ]
        }
        6 => {
            let val = n - offset!(6) as u128;
            let b = val.to_be_bytes();
            [
                0x04 | ((val >> 40) as u8),
                b[11],
                b[12],
                b[13],
                b[14],
                b[15],
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
            ]
        }
        7 => {
            let val = n - offset!(7) as u128;
            let b = val.to_be_bytes();
            [
                0x02 | ((val >> 48) as u8),
                b[10],
                b[11],
                b[12],
                b[13],
                b[14],
                b[15],
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
            ]
        }
        8 => {
            let val = n - offset!(8) as u128;
            let b = val.to_be_bytes();
            [
                0x01, b[9], b[10], b[11], b[12], b[13], b[14], b[15], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
        }
        9 => {
            let val = n - offset!(9) as u128;
            let b = val.to_be_bytes();
            [
                0x00, b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15], 0, 0, 0, 0, 0, 0, 0, 0,
                0,
            ]
        }
        10 => {
            let val = n - offset!(10);
            let b = val.to_be_bytes();
            [
                0x00,
                0x40 | ((val >> 64) as u8),
                b[8],
                b[9],
                b[10],
                b[11],
                b[12],
                b[13],
                b[14],
                b[15],
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]
        }
        11 => {
            let val = n - offset!(11);
            let b = val.to_be_bytes();
            [
                0x00,
                0x20 | ((val >> 72) as u8),
                b[7],
                b[8],
                b[9],
                b[10],
                b[11],
                b[12],
                b[13],
                b[14],
                b[15],
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]
        }
        12 => {
            let val = n - offset!(12);
            let b = val.to_be_bytes();
            [
                0x00,
                0x10 | ((val >> 80) as u8),
                b[6],
                b[7],
                b[8],
                b[9],
                b[10],
                b[11],
                b[12],
                b[13],
                b[14],
                b[15],
                0,
                0,
                0,
                0,
                0,
                0,
            ]
        }
        13 => {
            let val = n - offset!(13);
            let b = val.to_be_bytes();
            [
                0x00,
                0x08 | ((val >> 88) as u8),
                b[5],
                b[6],
                b[7],
                b[8],
                b[9],
                b[10],
                b[11],
                b[12],
                b[13],
                b[14],
                b[15],
                0,
                0,
                0,
                0,
                0,
            ]
        }
        14 => {
            let val = n - offset!(14);
            let b = val.to_be_bytes();
            [
                0x00,
                0x04 | ((val >> 96) as u8),
                b[4],
                b[5],
                b[6],
                b[7],
                b[8],
                b[9],
                b[10],
                b[11],
                b[12],
                b[13],
                b[14],
                b[15],
                0,
                0,
                0,
                0,
            ]
        }
        15 => {
            let val = n - offset!(15);
            let b = val.to_be_bytes();
            [
                0x00,
                0x02 | ((val >> 104) as u8),
                b[3],
                b[4],
                b[5],
                b[6],
                b[7],
                b[8],
                b[9],
                b[10],
                b[11],
                b[12],
                b[13],
                b[14],
                b[15],
                0,
                0,
                0,
            ]
        }
        16 => {
            let val = n - offset!(16);
            let b = val.to_be_bytes();
            [
                0x00, 0x01, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12],
                b[13], b[14], b[15], 0, 0,
            ]
        }
        _ => {
            let b = n.to_be_bytes();
            [
                0x00, 0x00, b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10],
                b[11], b[12], b[13], b[14], b[15],
            ]
        }
    };

    Vu128(out, PhantomData)
}

/// Encode a u128 in little-endian VLQ format.
#[inline(always)]
#[must_use]
pub const fn encode_vu128_le(n: u128) -> Vu128<LE> {
    let len = encode_len_vu128(n);

    let out: [u8; 18] = match len {
        1 => [
            0x80 | (n as u8),
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
            0,
            0,
            0,
        ],
        2 => {
            let val = n - offset!(2) as u128;
            let b = val.to_le_bytes();
            [
                0x40 | ((val >> 8) as u8),
                b[0],
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
                0,
                0,
            ]
        }
        3 => {
            let val = n - offset!(3) as u128;
            let b = val.to_le_bytes();
            [
                0x20 | ((val >> 16) as u8),
                b[0],
                b[1],
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
                0,
            ]
        }
        4 => {
            let val = n - offset!(4) as u128;
            let b = val.to_le_bytes();
            [
                0x10 | ((val >> 24) as u8),
                b[0],
                b[1],
                b[2],
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
            ]
        }
        5 => {
            let val = n - offset!(5) as u128;
            let b = val.to_le_bytes();
            [
                0x08 | ((val >> 32) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
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
            ]
        }
        6 => {
            let val = n - offset!(6) as u128;
            let b = val.to_le_bytes();
            [
                0x04 | ((val >> 40) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
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
            ]
        }
        7 => {
            let val = n - offset!(7) as u128;
            let b = val.to_le_bytes();
            [
                0x02 | ((val >> 48) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
                b[5],
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
            ]
        }
        8 => {
            let val = n - offset!(8) as u128;
            let b = val.to_le_bytes();
            [
                0x01, b[0], b[1], b[2], b[3], b[4], b[5], b[6], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
        }
        9 => {
            let val = n - offset!(9) as u128;
            let b = val.to_le_bytes();
            [
                0x00,
                (val >> 56) as u8,
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
                b[5],
                b[6],
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]
        }
        10 => {
            let val = n - offset!(10);
            let b = val.to_le_bytes();
            [
                0x00,
                0x40 | ((val >> 64) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
                b[5],
                b[6],
                b[7],
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]
        }
        11 => {
            let val = n - offset!(11);
            let b = val.to_le_bytes();
            [
                0x00,
                0x20 | ((val >> 72) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
                b[5],
                b[6],
                b[7],
                b[8],
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]
        }
        12 => {
            let val = n - offset!(12);
            let b = val.to_le_bytes();
            [
                0x00,
                0x10 | ((val >> 80) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
                b[5],
                b[6],
                b[7],
                b[8],
                b[9],
                0,
                0,
                0,
                0,
                0,
                0,
            ]
        }
        13 => {
            let val = n - offset!(13);
            let b = val.to_le_bytes();
            [
                0x00,
                0x08 | ((val >> 88) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
                b[5],
                b[6],
                b[7],
                b[8],
                b[9],
                b[10],
                0,
                0,
                0,
                0,
                0,
            ]
        }
        14 => {
            let val = n - offset!(14);
            let b = val.to_le_bytes();
            [
                0x00,
                0x04 | ((val >> 96) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
                b[5],
                b[6],
                b[7],
                b[8],
                b[9],
                b[10],
                b[11],
                0,
                0,
                0,
                0,
            ]
        }
        15 => {
            let val = n - offset!(15);
            let b = val.to_le_bytes();
            [
                0x00,
                0x02 | ((val >> 104) as u8),
                b[0],
                b[1],
                b[2],
                b[3],
                b[4],
                b[5],
                b[6],
                b[7],
                b[8],
                b[9],
                b[10],
                b[11],
                b[12],
                0,
                0,
                0,
            ]
        }
        16 => {
            let val = n - offset!(16);
            let b = val.to_le_bytes();
            [
                0x00, 0x01, b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10],
                b[11], b[12], b[13], 0, 0,
            ]
        }
        _ => {
            let b = n.to_le_bytes();
            [
                0x00, 0x00, b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10],
                b[11], b[12], b[13], b[14], b[15],
            ]
        }
    };

    Vu128(out, PhantomData)
}

/// Decode a big-endian VLQ back to u128.
#[inline(always)]
pub const fn decode_vu128_be(n: Vu128<BE>) -> u128 {
    let len = n.len();
    let b = n.bytes();

    // Load first 8 bytes as u64 for cases 1-8
    let raw = u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);

    match len {
        1 => ((raw >> 56) & 0x7F) as u128,
        2 => (((raw >> 48) & 0x3FFF) as u128) + offset!(2) as u128,
        3 => (((raw >> 40) & 0x1F_FFFF) as u128) + offset!(3) as u128,
        4 => (((raw >> 32) & 0x0FFF_FFFF) as u128) + offset!(4) as u128,
        5 => (((raw >> 24) & 0x07_FFFF_FFFF) as u128) + offset!(5) as u128,
        6 => (((raw >> 16) & 0x03FF_FFFF_FFFF) as u128) + offset!(6) as u128,
        7 => (((raw >> 8) & 0x01_FFFF_FFFF_FFFF) as u128) + offset!(7) as u128,
        8 => ((raw & 0x00FF_FFFF_FFFF_FFFF) as u128) + offset!(8) as u128,
        9 => {
            (u64::from_be_bytes([b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8]]) as u128)
                + offset!(9) as u128
        }
        10 => {
            let data = u128::from_be_bytes([0, 0, 0, 0, 0, 0, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], 0, 0]);
            ((((b[1] & 0x3F) as u128) << 64) | (data >> 16)) + offset!(10)
        }
        11 => {
            let data = u128::from_be_bytes([0, 0, 0, 0, 0, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], 0, 0]);
            ((((b[1] & 0x1F) as u128) << 72) | (data >> 16)) + offset!(11)
        }
        12 => {
            let data = u128::from_be_bytes([0, 0, 0, 0, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], 0, 0]);
            ((((b[1] & 0x0F) as u128) << 80) | (data >> 16)) + offset!(12)
        }
        13 => {
            let data = u128::from_be_bytes([0, 0, 0, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], 0, 0]);
            ((((b[1] & 0x07) as u128) << 88) | (data >> 16)) + offset!(13)
        }
        14 => {
            let data = u128::from_be_bytes([0, 0, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], 0, 0]);
            ((((b[1] & 0x03) as u128) << 96) | (data >> 16)) + offset!(14)
        }
        15 => {
            let data = u128::from_be_bytes([0, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], 0, 0]);
            ((((b[1] & 0x01) as u128) << 104) | (data >> 16)) + offset!(15)
        }
        16 => {
            let data = u128::from_be_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15], 0, 0]);
            (data >> 16) + offset!(16)
        }
        _ => {
            u128::from_be_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15], b[16], b[17]])
        }
    }
}

/// Decode a little-endian VLQ back to u128.
#[inline(always)]
pub const fn decode_vu128_le(n: Vu128<LE>) -> u128 {
    let len = n.len();
    let b = n.bytes();

    // Data bytes (after prefix) are in LE order starting at b[1]
    let data = u64::from_le_bytes([b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8]]);

    match len {
        1 => (b[0] & 0x7F) as u128,
        2 => ((((b[0] & 0x3F) as u128) << 8) | (data & 0xFF) as u128) + offset!(2) as u128,
        3 => ((((b[0] & 0x1F) as u128) << 16) | (data & 0xFFFF) as u128) + offset!(3) as u128,
        4 => ((((b[0] & 0x0F) as u128) << 24) | (data & 0xFF_FFFF) as u128) + offset!(4) as u128,
        5 => ((((b[0] & 0x07) as u128) << 32) | (data & 0xFFFF_FFFF) as u128) + offset!(5) as u128,
        6 => ((((b[0] & 0x03) as u128) << 40) | (data & 0xFF_FFFF_FFFF) as u128) + offset!(6) as u128,
        7 => ((((b[0] & 0x01) as u128) << 48) | (data & 0xFFFF_FFFF_FFFF) as u128) + offset!(7) as u128,
        8 => ((data & 0xFF_FFFF_FFFF_FFFF) as u128) + offset!(8) as u128,
        9 => {
            let high = (b[1] as u128) << 56;
            let low = u64::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], 0]) as u128;
            (high | low) + offset!(9) as u128
        }
        10 => {
            let lo = u128::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], 0, 0, 0, 0, 0, 0, 0, 0]);
            ((((b[1] & 0x3F) as u128) << 64) | lo) + offset!(10)
        }
        11 => {
            let lo = u128::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], 0, 0, 0, 0, 0, 0, 0]);
            ((((b[1] & 0x1F) as u128) << 72) | lo) + offset!(11)
        }
        12 => {
            let lo = u128::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], 0, 0, 0, 0, 0, 0]);
            ((((b[1] & 0x0F) as u128) << 80) | lo) + offset!(12)
        }
        13 => {
            let lo = u128::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], 0, 0, 0, 0, 0]);
            ((((b[1] & 0x07) as u128) << 88) | lo) + offset!(13)
        }
        14 => {
            let lo = u128::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], 0, 0, 0, 0]);
            ((((b[1] & 0x03) as u128) << 96) | lo) + offset!(14)
        }
        15 => {
            let lo = u128::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], 0, 0, 0]);
            ((((b[1] & 0x01) as u128) << 104) | lo) + offset!(15)
        }
        16 => {
            let lo = u128::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15], 0, 0]);
            lo + offset!(16)
        }
        _ => {
            u128::from_le_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15], b[16], b[17]])
        }
    }
}

/// An unsigned 128-bit integer in variable-length quantity encoding.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vu128<E>(pub(crate) [u8; VU128_BUF_SIZE], pub(crate) PhantomData<E>);

#[allow(clippy::len_without_is_empty)]
impl<E> Vu128<E> {
    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu128(self.0[0], self.0[1])
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 18] {
        self.0
    }
}

impl Vu128<BE> {
    /// Construct a new big-endian VLQ instance from the given `u128`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u128) -> Vu128<BE> {
        encode_vu128_be(value)
    }

    /// Retrieve the stored number as `u128`.
    #[inline(always)]
    pub const fn get(&self) -> u128 {
        decode_vu128_be(*self)
    }
}

impl Vu128<LE> {
    /// Construct a new little-endian VLQ instance from the given `u128`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u128) -> Vu128<LE> {
        encode_vu128_le(value)
    }

    /// Retrieve the stored number as `u128`.
    #[inline(always)]
    pub const fn get(&self) -> u128 {
        decode_vu128_le(*self)
    }
}

impl From<u128> for Vu128<BE> {
    fn from(n: u128) -> Self {
        encode_vu128_be(n)
    }
}

impl From<u128> for Vu128<LE> {
    fn from(n: u128) -> Self {
        encode_vu128_le(n)
    }
}

impl From<Vu128<BE>> for u128 {
    fn from(n: Vu128<BE>) -> Self {
        decode_vu128_be(n)
    }
}

impl From<Vu128<LE>> for u128 {
    fn from(n: Vu128<LE>) -> Self {
        decode_vu128_le(n)
    }
}

impl<E> Display for Vu128<E>
where
    Vu128<E>: Copy,
    u128: From<Vu128<E>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&u128::from(*self), f)
    }
}

impl<E> Debug for Vu128<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vu128(0b")?;
        for x in self.0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0[len]))
    }
}
