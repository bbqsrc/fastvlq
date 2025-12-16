//! Unsigned 32-bit VLQ encoding.

use core::fmt::{Debug, Display};
use core::marker::PhantomData;

use crate::{BE, LE};

pub(crate) const VU32_BUF_SIZE: usize = 5;

/// Decode length from first byte for u32 (max 5 bytes).
#[inline(always)]
pub(crate) const fn decode_len_vu32(n: u8) -> u8 {
    let len = n.leading_zeros() as u8 + 1;
    if len > 5 { 5 } else { len }
}

/// Encode a u32 in big-endian VLQ format.
#[inline(always)]
#[must_use]
pub const fn encode_vu32_be(n: u32) -> Vu32<BE> {
    let n64 = n as u64;

    // Pack prefix + value into u64, left-aligned
    let packed: u64 = if n64 < offset!(2) as u64 {
        (0x80 | n64) << 56
    } else if n64 < offset!(3) as u64 {
        (0x4000 | (n64 - offset!(2) as u64)) << 48
    } else if n64 < offset!(4) as u64 {
        (0x20_0000 | (n64 - offset!(3) as u64)) << 40
    } else if n64 < offset!(5) {
        (0x1000_0000 | (n64 - offset!(4) as u64)) << 32
    } else {
        (0x08_0000_0000 | (n64 - offset!(5))) << 24
    };

    let b = packed.to_be_bytes();
    Vu32([b[0], b[1], b[2], b[3], b[4]], PhantomData)
}

/// Encode a u32 in little-endian VLQ format.
#[inline(always)]
#[must_use]
pub const fn encode_vu32_le(n: u32) -> Vu32<LE> {
    let n64 = n as u64;

    let out = if n64 < offset!(2) as u64 {
        [0x80 | (n as u8), 0, 0, 0, 0]
    } else if n64 < offset!(3) as u64 {
        let val = n64 - offset!(2) as u64;
        let b = val.to_le_bytes();
        [0x40 | ((val >> 8) as u8), b[0], 0, 0, 0]
    } else if n64 < offset!(4) as u64 {
        let val = n64 - offset!(3) as u64;
        let b = val.to_le_bytes();
        [0x20 | ((val >> 16) as u8), b[0], b[1], 0, 0]
    } else if n64 < offset!(5) {
        let val = n64 - offset!(4) as u64;
        let b = val.to_le_bytes();
        [0x10 | ((val >> 24) as u8), b[0], b[1], b[2], 0]
    } else {
        let val = n64 - offset!(5);
        let b = val.to_le_bytes();
        [0x08 | ((val >> 32) as u8), b[0], b[1], b[2], b[3]]
    };

    Vu32(out, PhantomData)
}

/// Decode a big-endian VLQ back to u32.
#[inline(always)]
pub const fn decode_vu32_be(n: Vu32<BE>) -> u32 {
    let len = n.len();
    let b = n.bytes();
    let raw = u64::from_be_bytes([0, 0, 0, b[0], b[1], b[2], b[3], b[4]]);

    match len {
        1 => ((raw >> 32) & 0x7F) as u32,
        2 => (((raw >> 24) & 0x3FFF) as u32) + offset!(2) as u32,
        3 => (((raw >> 16) & 0x1F_FFFF) as u32) + offset!(3) as u32,
        4 => (((raw >> 8) & 0x0FFF_FFFF) as u32) + offset!(4) as u32,
        _ => ((raw & 0x07_FFFF_FFFF) + offset!(5)) as u32,
    }
}

/// Decode a little-endian VLQ back to u32.
#[inline(always)]
pub const fn decode_vu32_le(n: Vu32<LE>) -> u32 {
    let len = n.len();
    let b = n.bytes();
    let data = u64::from_le_bytes([b[1], b[2], b[3], b[4], 0, 0, 0, 0]);

    match len {
        1 => (b[0] & 0x7F) as u32,
        2 => ((((b[0] & 0x3F) as u32) << 8) | (data & 0xFF) as u32) + offset!(2) as u32,
        3 => ((((b[0] & 0x1F) as u32) << 16) | (data & 0xFFFF) as u32) + offset!(3) as u32,
        4 => ((((b[0] & 0x0F) as u32) << 24) | (data & 0xFF_FFFF) as u32) + offset!(4) as u32,
        _ => ((((b[0] & 0x07) as u64) << 32) | (data & 0xFFFF_FFFF) + offset!(5)) as u32,
    }
}

/// An unsigned 32-bit integer in variable-length quantity encoding.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vu32<E>(pub(crate) [u8; VU32_BUF_SIZE], pub(crate) PhantomData<E>);

#[allow(clippy::len_without_is_empty)]
impl<E> Vu32<E> {
    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu32(self.0[0])
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 5] {
        self.0
    }
}

impl Vu32<BE> {
    /// Construct a new big-endian VLQ instance from the given `u32`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u32) -> Vu32<BE> {
        encode_vu32_be(value)
    }

    /// Retrieve the stored number as `u32`.
    #[inline(always)]
    pub const fn get(&self) -> u32 {
        decode_vu32_be(*self)
    }
}

impl Vu32<LE> {
    /// Construct a new little-endian VLQ instance from the given `u32`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u32) -> Vu32<LE> {
        encode_vu32_le(value)
    }

    /// Retrieve the stored number as `u32`.
    #[inline(always)]
    pub const fn get(&self) -> u32 {
        decode_vu32_le(*self)
    }
}

impl From<u32> for Vu32<BE> {
    fn from(n: u32) -> Self {
        encode_vu32_be(n)
    }
}

impl From<u32> for Vu32<LE> {
    fn from(n: u32) -> Self {
        encode_vu32_le(n)
    }
}

impl From<Vu32<BE>> for u32 {
    fn from(n: Vu32<BE>) -> Self {
        decode_vu32_be(n)
    }
}

impl From<Vu32<LE>> for u32 {
    fn from(n: Vu32<LE>) -> Self {
        decode_vu32_le(n)
    }
}

impl<E> Display for Vu32<E>
where
    Vu32<E>: Copy,
    u32: From<Vu32<E>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&u32::from(*self), f)
    }
}

impl<E> Debug for Vu32<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vu32(0b")?;
        for x in self.0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0[len]))
    }
}
