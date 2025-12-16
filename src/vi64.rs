//! Signed 64-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};
use core::marker::PhantomData;

use crate::vu64::{Vu64, VU64_BUF_SIZE, decode_vu64_be, decode_vu64_le, encode_vu64_be, encode_vu64_le};
use crate::{BE, LE};

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

/// Encode a signed i64 using zigzag encoding to big-endian VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi64_be(n: i64) -> Vi64<BE> {
    Vi64(encode_vu64_be(zigzag_encode_i64(n)), PhantomData)
}

/// Encode a signed i64 using zigzag encoding to little-endian VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi64_le(n: i64) -> Vi64<LE> {
    Vi64(encode_vu64_le(zigzag_encode_i64(n)), PhantomData)
}

/// Decode a big-endian Vi64 back to a native i64.
#[inline(always)]
pub fn decode_vi64_be(n: Vi64<BE>) -> i64 {
    zigzag_decode_i64(decode_vu64_be(n.0))
}

/// Decode a little-endian Vi64 back to a native i64.
#[inline(always)]
pub fn decode_vi64_le(n: Vi64<LE>) -> i64 {
    zigzag_decode_i64(decode_vu64_le(n.0))
}

/// A signed 64-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi64<E>(Vu64<E>, PhantomData<E>);

#[allow(clippy::len_without_is_empty)]
impl<E> Vi64<E> {
    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.0.len()
    }
}

impl Vi64<BE> {
    /// Construct a new big-endian VLQ instance from the given `i64`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: i64) -> Vi64<BE> {
        encode_vi64_be(value)
    }

    /// Retrieve the stored number as `i64`.
    #[inline(always)]
    pub fn get(&self) -> i64 {
        decode_vi64_be(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; VU64_BUF_SIZE] {
        self.0.bytes()
    }
}

impl Vi64<LE> {
    /// Construct a new little-endian VLQ instance from the given `i64`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: i64) -> Vi64<LE> {
        encode_vi64_le(value)
    }

    /// Retrieve the stored number as `i64`.
    #[inline(always)]
    pub fn get(&self) -> i64 {
        decode_vi64_le(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; VU64_BUF_SIZE] {
        self.0.bytes()
    }
}

impl From<i64> for Vi64<BE> {
    fn from(n: i64) -> Self {
        encode_vi64_be(n)
    }
}

impl From<i64> for Vi64<LE> {
    fn from(n: i64) -> Self {
        encode_vi64_le(n)
    }
}

impl From<Vi64<BE>> for i64 {
    fn from(n: Vi64<BE>) -> Self {
        decode_vi64_be(n)
    }
}

impl From<Vi64<LE>> for i64 {
    fn from(n: Vi64<LE>) -> Self {
        decode_vi64_le(n)
    }
}

impl<E> Display for Vi64<E>
where
    Vi64<E>: Copy,
    i64: From<Vi64<E>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&i64::from(*self), f)
    }
}

impl Vi64<BE> {
    fn fmt_debug(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vi64(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}

impl Vi64<LE> {
    fn fmt_debug(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vi64(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}

impl Debug for Vi64<BE> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.fmt_debug(f)
    }
}

impl Debug for Vi64<LE> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.fmt_debug(f)
    }
}
