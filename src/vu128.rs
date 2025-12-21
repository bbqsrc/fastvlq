//! Unsigned 128-bit VLQ encoding.
//!
//! Format:
//! - First byte != 0x00: vu64 for lo (1-8 bytes), hi = 0
//! - First byte == 0x00: 0x00 + lo raw (8 bytes) + vu64(hi) = 10-18 bytes

use core::fmt::{Debug, Display};

use crate::vu64::{decode_len_vu64, decode_vu64_slice, encode_vu64};

pub(crate) const VU128_BUF_SIZE: usize = 18;

/// Decode length from buffer for u128.
#[inline(always)]
pub(crate) fn decode_len_vu128(data: &[u8]) -> u8 {
    if data[0] != 0x00 {
        decode_len_vu64(data[0]) // 1-8 bytes
    } else {
        9 + decode_len_vu64(data[9]) // 9 + hi_len
    }
}

/// Encode a u128 to VLQ format.
#[inline(always)]
pub fn encode_vu128(n: u128) -> Vu128 {
    let lo = n as u64;
    let hi = (n >> 64) as u64;
    let mut buf = [0u8; VU128_BUF_SIZE];

    // Try compact format: vu64(lo) if hi=0 and lo fits in 1-8 bytes
    if hi == 0 {
        let lo_enc = encode_vu64(lo);
        if lo_enc.bytes()[0] != 0x00 {
            // Compact: 1-8 bytes
            buf[..lo_enc.len() as usize].copy_from_slice(lo_enc.bytes());
            return Vu128(buf);
        }
    }

    // Extended: 0x00 + lo raw (8 bytes) + vu64(hi)
    buf[0] = 0x00;
    buf[1..9].copy_from_slice(&lo.to_le_bytes());
    let hi_enc = encode_vu64(hi);
    buf[9..9 + hi_enc.len() as usize].copy_from_slice(hi_enc.bytes());

    Vu128(buf)
}

/// Decode a u128 from a Vu128.
#[inline(always)]
pub fn decode_vu128(n: Vu128) -> u128 {
    n.get()
}

/// Decode a u128 from a byte slice.
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[inline(always)]
pub fn decode_vu128_slice(data: &[u8]) -> (u128, usize) {
    if data.is_empty() {
        return (0, 0);
    }

    if data[0] != 0x00 {
        // Compact: vu64 for lo, hi = 0
        let (lo, len) = decode_vu64_slice(data);
        return (lo as u128, len);
    }

    // Extended: 0x00 + lo raw (8 bytes) + vu64(hi)
    if data.len() < 9 {
        return (0, 0);
    }
    let lo = u64::from_le_bytes(data[1..9].try_into().unwrap());

    let (hi, hi_len) = decode_vu64_slice(&data[9..]);
    if hi_len == 0 {
        return (0, 0);
    }

    (((hi as u128) << 64) | (lo as u128), 9 + hi_len)
}

/// An unsigned 128-bit integer in variable-length quantity encoding.
#[derive(Clone, Copy)]
pub struct Vu128(pub(crate) [u8; VU128_BUF_SIZE]);

#[allow(clippy::len_without_is_empty)]
impl Vu128 {
    /// Construct a new VLQ instance from the given `u128`.
    #[inline(always)]
    pub fn new(value: u128) -> Vu128 {
        encode_vu128(value)
    }

    /// Retrieve the stored number as `u128`.
    #[inline(always)]
    pub fn get(&self) -> u128 {
        decode_vu128_slice(&self.0).0
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub fn len(&self) -> u8 {
        decode_len_vu128(&self.0)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub fn bytes(&self) -> &[u8] {
        &self.0[..self.len() as usize]
    }
}

impl From<u128> for Vu128 {
    fn from(n: u128) -> Self {
        encode_vu128(n)
    }
}

impl From<Vu128> for u128 {
    fn from(n: Vu128) -> Self {
        n.get()
    }
}

impl Display for Vu128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&u128::from(*self), f)
    }
}

impl Debug for Vu128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vu128(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
