//! Signed 32-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};

use crate::vu32::{decode_vu32, encode_vu32, Vu32};

#[inline(always)]
pub(crate) const fn zigzag_encode_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

#[inline(always)]
pub(crate) const fn zigzag_decode_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ -((n & 1) as i32)
}

/// Encode a signed i32 using zigzag encoding to VLQ.
#[inline(always)]
#[must_use]
pub const fn encode_vi32(n: i32) -> Vi32 {
    Vi32(encode_vu32(zigzag_encode_i32(n)))
}

/// Decode a Vi32 back to a native i32.
#[inline(always)]
pub const fn decode_vi32(n: Vi32) -> i32 {
    zigzag_decode_i32(decode_vu32(n.0))
}

/// A signed 32-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi32(Vu32);

#[allow(clippy::len_without_is_empty)]
impl Vi32 {
    /// Construct a new VLQ instance from the given `i32`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: i32) -> Vi32 {
        encode_vi32(value)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.0.len()
    }

    /// Retrieve the stored number as `i32`.
    #[inline(always)]
    pub const fn get(&self) -> i32 {
        decode_vi32(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 5] {
        self.0.bytes()
    }

    /// Get the serialized representation of the VLQ as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<i32> for Vi32 {
    fn from(n: i32) -> Self {
        encode_vi32(n)
    }
}

impl From<Vi32> for i32 {
    fn from(n: Vi32) -> Self {
        decode_vi32(n)
    }
}

impl Display for Vi32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl Debug for Vi32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vi32(0b")?;
        for x in self.0 .0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0 .0[len]))
    }
}
