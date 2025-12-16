//! Signed 32-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};
use core::marker::PhantomData;

use crate::vu32::{Vu32, decode_vu32_be, decode_vu32_le, encode_vu32_be, encode_vu32_le};
use crate::{BE, LE};

/// Zigzag encode a signed i32 to unsigned u32.
#[inline(always)]
pub const fn zigzag_encode_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

/// Zigzag decode an unsigned u32 to signed i32.
#[inline(always)]
pub const fn zigzag_decode_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ -((n & 1) as i32)
}

/// Encode a signed i32 using zigzag encoding to big-endian VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi32_be(n: i32) -> Vi32<BE> {
    Vi32(encode_vu32_be(zigzag_encode_i32(n)), PhantomData)
}

/// Encode a signed i32 using zigzag encoding to little-endian VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi32_le(n: i32) -> Vi32<LE> {
    Vi32(encode_vu32_le(zigzag_encode_i32(n)), PhantomData)
}

/// Decode a big-endian Vi32 back to a native i32.
#[inline(always)]
pub const fn decode_vi32_be(n: Vi32<BE>) -> i32 {
    zigzag_decode_i32(decode_vu32_be(n.0))
}

/// Decode a little-endian Vi32 back to a native i32.
#[inline(always)]
pub const fn decode_vi32_le(n: Vi32<LE>) -> i32 {
    zigzag_decode_i32(decode_vu32_le(n.0))
}

/// A signed 32-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi32<E>(Vu32<E>, PhantomData<E>);

#[allow(clippy::len_without_is_empty)]
impl<E> Vi32<E> {
    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.0.len()
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 5] {
        self.0.bytes()
    }
}

impl Vi32<BE> {
    /// Construct a new big-endian VLQ instance from the given `i32`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: i32) -> Vi32<BE> {
        encode_vi32_be(value)
    }

    /// Retrieve the stored number as `i32`.
    #[inline(always)]
    pub const fn get(&self) -> i32 {
        decode_vi32_be(*self)
    }
}

impl Vi32<LE> {
    /// Construct a new little-endian VLQ instance from the given `i32`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: i32) -> Vi32<LE> {
        encode_vi32_le(value)
    }

    /// Retrieve the stored number as `i32`.
    #[inline(always)]
    pub const fn get(&self) -> i32 {
        decode_vi32_le(*self)
    }
}

impl From<i32> for Vi32<BE> {
    fn from(n: i32) -> Self {
        encode_vi32_be(n)
    }
}

impl From<i32> for Vi32<LE> {
    fn from(n: i32) -> Self {
        encode_vi32_le(n)
    }
}

impl From<Vi32<BE>> for i32 {
    fn from(n: Vi32<BE>) -> Self {
        decode_vi32_be(n)
    }
}

impl From<Vi32<LE>> for i32 {
    fn from(n: Vi32<LE>) -> Self {
        decode_vi32_le(n)
    }
}

impl<E> Display for Vi32<E>
where
    Vi32<E>: Copy,
    i32: From<Vi32<E>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&i32::from(*self), f)
    }
}

impl<E> Debug for Vi32<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.0.bytes();
        write!(f, "Vi32(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
