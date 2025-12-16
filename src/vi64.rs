//! Signed 64-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};

use crate::vu64::{VU64_BUF_SIZE, Vu64, decode_vu64, encode_vu64};

/// Zigzag encode a signed i64 to unsigned u64.
#[inline(always)]
pub const fn zigzag_encode_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// Zigzag decode an unsigned u64 to signed i64.
#[inline(always)]
pub const fn zigzag_decode_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ -((n & 1) as i64)
}

/// Encode a signed i64 using zigzag encoding to VLQ.
#[inline(always)]
pub fn encode_vi64(n: i64) -> Vi64 {
    Vi64(encode_vu64(zigzag_encode_i64(n)))
}

/// Decode a Vi64 back to a native i64.
#[inline(always)]
pub fn decode_vi64(n: Vi64) -> i64 {
    zigzag_decode_i64(decode_vu64(n.0))
}

/// A signed 64-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi64(Vu64);

#[allow(clippy::len_without_is_empty)]
impl Vi64 {
    /// Construct a new VLQ instance from the given `i64`.
    #[inline(always)]
    pub fn new(value: i64) -> Vi64 {
        encode_vi64(value)
    }

    /// Retrieve the stored number as `i64`.
    #[inline(always)]
    pub fn get(&self) -> i64 {
        decode_vi64(*self)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.0.len()
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; VU64_BUF_SIZE] {
        self.0.bytes()
    }
}

impl From<i64> for Vi64 {
    fn from(n: i64) -> Self {
        encode_vi64(n)
    }
}

impl From<Vi64> for i64 {
    fn from(n: Vi64) -> Self {
        decode_vi64(n)
    }
}

impl Display for Vi64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&i64::from(*self), f)
    }
}

impl Debug for Vi64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vi64(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
