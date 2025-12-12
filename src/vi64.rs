//! Signed 64-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};

use crate::vu64::{Vu64, decode_vu64, encode_vu64};

#[inline(always)]
pub(crate) const fn zigzag_encode_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

#[inline(always)]
pub(crate) const fn zigzag_decode_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ -((n & 1) as i64)
}

/// Encode a signed i64 using zigzag encoding to VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi64(n: i64) -> Vi64 {
    Vi64(encode_vu64(zigzag_encode_i64(n)))
}

/// Decode a Vi64 back to a native i64.
#[inline(always)]
pub const fn decode_vi64(n: Vi64) -> i64 {
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
    #[must_use]
    pub const fn new(value: i64) -> Vi64 {
        encode_vi64(value)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.0.len()
    }

    /// Retrieve the stored number as `i64`.
    #[inline(always)]
    pub const fn get(&self) -> i64 {
        decode_vi64(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 9] {
        self.0.bytes()
    }

    /// Get the serialized representation of the VLQ as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
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
        Display::fmt(&self.get(), f)
    }
}

impl Debug for Vi64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vi64(0b")?;
        for x in self.0.0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0.0[len]))
    }
}
