//! Signed 128-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};

use crate::vu128::{Vu128, decode_vu128, encode_vu128};

#[inline(always)]
pub(crate) const fn zigzag_encode_i128(n: i128) -> u128 {
    ((n << 1) ^ (n >> 127)) as u128
}

#[inline(always)]
pub(crate) const fn zigzag_decode_i128(n: u128) -> i128 {
    ((n >> 1) as i128) ^ -((n & 1) as i128)
}

/// Encode a signed i128 using zigzag encoding to VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi128(n: i128) -> Vi128 {
    Vi128(encode_vu128(zigzag_encode_i128(n)))
}

/// Decode a Vi128 back to a native i128.
#[inline(always)]
pub const fn decode_vi128(n: Vi128) -> i128 {
    zigzag_decode_i128(decode_vu128(n.0))
}

/// A signed 128-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi128(Vu128);

#[allow(clippy::len_without_is_empty)]
impl Vi128 {
    /// Construct a new VLQ instance from the given `i128`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: i128) -> Vi128 {
        encode_vi128(value)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.0.len()
    }

    /// Retrieve the stored number as `i128`.
    #[inline(always)]
    pub const fn get(&self) -> i128 {
        decode_vi128(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 18] {
        self.0.bytes()
    }

    /// Get the serialized representation of the VLQ as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<i128> for Vi128 {
    fn from(n: i128) -> Self {
        encode_vi128(n)
    }
}

impl From<Vi128> for i128 {
    fn from(n: Vi128) -> Self {
        decode_vi128(n)
    }
}

impl Display for Vi128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl Debug for Vi128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vi128(0b")?;
        for x in self.0.0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0.0[len]))
    }
}
