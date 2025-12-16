//! Signed 128-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};
use core::marker::PhantomData;

use crate::vu128::{Vu128, decode_vu128_be, decode_vu128_le, encode_vu128_be, encode_vu128_le};
use crate::{BE, LE};

/// Zigzag encode a signed i128 to unsigned u128.
#[inline(always)]
pub const fn zigzag_encode_i128(n: i128) -> u128 {
    ((n << 1) ^ (n >> 127)) as u128
}

/// Zigzag decode an unsigned u128 to signed i128.
#[inline(always)]
pub const fn zigzag_decode_i128(n: u128) -> i128 {
    ((n >> 1) as i128) ^ -((n & 1) as i128)
}

/// Encode a signed i128 using zigzag encoding to big-endian VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi128_be(n: i128) -> Vi128<BE> {
    Vi128(encode_vu128_be(zigzag_encode_i128(n)), PhantomData)
}

/// Encode a signed i128 using zigzag encoding to little-endian VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi128_le(n: i128) -> Vi128<LE> {
    Vi128(encode_vu128_le(zigzag_encode_i128(n)), PhantomData)
}

/// Decode a big-endian Vi128 back to a native i128.
#[inline(always)]
pub const fn decode_vi128_be(n: Vi128<BE>) -> i128 {
    zigzag_decode_i128(decode_vu128_be(n.0))
}

/// Decode a little-endian Vi128 back to a native i128.
#[inline(always)]
pub const fn decode_vi128_le(n: Vi128<LE>) -> i128 {
    zigzag_decode_i128(decode_vu128_le(n.0))
}

/// A signed 128-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi128<E>(Vu128<E>, PhantomData<E>);

#[allow(clippy::len_without_is_empty)]
impl<E> Vi128<E> {
    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.0.len()
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 18] {
        self.0.bytes()
    }
}

impl Vi128<BE> {
    /// Construct a new big-endian VLQ instance from the given `i128`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: i128) -> Vi128<BE> {
        encode_vi128_be(value)
    }

    /// Retrieve the stored number as `i128`.
    #[inline(always)]
    pub const fn get(&self) -> i128 {
        decode_vi128_be(*self)
    }
}

impl Vi128<LE> {
    /// Construct a new little-endian VLQ instance from the given `i128`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: i128) -> Vi128<LE> {
        encode_vi128_le(value)
    }

    /// Retrieve the stored number as `i128`.
    #[inline(always)]
    pub const fn get(&self) -> i128 {
        decode_vi128_le(*self)
    }
}

impl From<i128> for Vi128<BE> {
    fn from(n: i128) -> Self {
        encode_vi128_be(n)
    }
}

impl From<i128> for Vi128<LE> {
    fn from(n: i128) -> Self {
        encode_vi128_le(n)
    }
}

impl From<Vi128<BE>> for i128 {
    fn from(n: Vi128<BE>) -> Self {
        decode_vi128_be(n)
    }
}

impl From<Vi128<LE>> for i128 {
    fn from(n: Vi128<LE>) -> Self {
        decode_vi128_le(n)
    }
}

impl<E> Display for Vi128<E>
where
    Vi128<E>: Copy,
    i128: From<Vi128<E>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&i128::from(*self), f)
    }
}

impl<E> Debug for Vi128<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.0.bytes();
        write!(f, "Vi128(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
