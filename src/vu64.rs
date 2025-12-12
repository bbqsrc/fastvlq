//! Unsigned 64-bit VLQ encoding.

use core::fmt::{Debug, Display};

pub(crate) const VU64_BUF_SIZE: usize = 9;

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
pub(crate) const fn decode_len_vu64(n: u8) -> u8 {
    n.leading_zeros() as u8 + 1
}

/// Encoding bit depth by length in bytes:
///
/// 1: 7 bits
/// 2: 14 (6 + 8) bits
/// 3: 21 (5 + 8 * 2) bits
/// 4: 28 (4 + 8 * 3) bits
/// 5: 35 (3 + 8 * 4) bits
/// 6: 42 (2 + 8 * 5) bits
/// 7: 49 (1 + 8 * 6) bits
/// 8: 56 (0 + 8 * 7) bits
/// 9: 64 (0 + 8 * 8) bits
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

/// Encode a u64 in value-length quantity encoding.
#[inline(always)]
#[must_use]
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

/// Decode a given VLQ instance back into a native u64.
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

/// An unsigned 64-bit integer in value-length quantity encoding.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vu64(pub(crate) [u8; VU64_BUF_SIZE]);

#[allow(clippy::len_without_is_empty)]
impl Vu64 {
    /// Construct a new VLQ instance from the given `u64`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u64) -> Vu64 {
        encode_vu64(value)
    }

    /// Length of the internal representation
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu64(self.0[0])
    }

    /// Retrieve the stored number as `u64`.
    #[inline(always)]
    pub const fn get(&self) -> u64 {
        decode_vu64(*self)
    }

    /// Get the raw byte representation of the VLQ instance
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 9] {
        self.0
    }

    /// Get the serialized representation of the VLQ as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.0[..(self.len() as usize)]
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

impl Display for Vu64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl Debug for Vu64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vu64(0b")?;
        for x in self.0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0[len]))
    }
}
