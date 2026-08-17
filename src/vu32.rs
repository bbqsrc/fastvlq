//! Unsigned 32-bit VLQ encoding.

use core::fmt::{Debug, Display};

#[cfg(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
))]
use crate::vu64::{decode_vu64_slice, encode_vu64};

pub(crate) const VU32_BUF_SIZE: usize = 5;

/// Decode length from first byte for u32 (max 5 bytes).
#[inline(always)]
pub(crate) const fn decode_len_vu32(n: u8) -> u8 {
    let len = n.leading_zeros() as u8 + 1;
    if len > 5 { 5 } else { len }
}

/// Encode a u32 in VLQ format.
#[inline(always)]
pub fn encode_vu32(n: u32) -> Vu32 {
    #[cfg(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
    ))]
    {
        // Use the u64 encoder and copy the first 5 bytes
        let encoded = encode_vu64(n as u64);
        let src = encoded.0;
        Vu32([src[0], src[1], src[2], src[3], src[4]])
    }

    #[cfg(not(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
    )))]
    {
        let n64 = n as u64;

        if n64 < offset!(2) as u64 {
            // len=1: all data in prefix
            Vu32([0x80 | (n as u8), 0, 0, 0, 0])
        } else if n64 < offset!(3) as u64 {
            // len=2: 1 data byte
            let val = n64 - offset!(2) as u64;
            Vu32([0x40 | ((val >> 8) as u8), val as u8, 0, 0, 0])
        } else if n64 < offset!(4) as u64 {
            // len=3: 2 data bytes
            let val = n64 - offset!(3) as u64;
            Vu32([
                0x20 | ((val >> 16) as u8),
                val as u8,
                (val >> 8) as u8,
                0,
                0,
            ])
        } else if n64 < offset!(5) {
            // len=4: 3 data bytes
            let val = n64 - offset!(4) as u64;
            Vu32([
                0x10 | ((val >> 24) as u8),
                val as u8,
                (val >> 8) as u8,
                (val >> 16) as u8,
                0,
            ])
        } else {
            // len=5: 4 data bytes
            let val = n64 - offset!(5);
            Vu32([
                0x08 | ((val >> 32) as u8),
                val as u8,
                (val >> 8) as u8,
                (val >> 16) as u8,
                (val >> 24) as u8,
            ])
        }
    }
}

/// Decode a u32 from a Vu32.
#[inline(always)]
pub fn decode_vu32(n: Vu32) -> u32 {
    n.get()
}

/// Decode a u32 from a byte slice.
/// Returns (0, 0) for empty or invalid input.
#[inline(always)]
pub fn decode_vu32_slice(data: &[u8]) -> (u32, usize) {
    #[cfg(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
    ))]
    {
        let (value, len) = decode_vu64_slice(data);
        (value as u32, len)
    }

    #[cfg(not(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
    )))]
    {
        let Some(&p) = data.first() else {
            return (0, 0);
        };

        // len=1: prefix >= 0x80 (1xxx_xxxx)
        if p >= 0x80 {
            return ((p & 0x7F) as u32, 1);
        }

        // len=2: prefix >= 0x40 (01xx_xxxx)
        if p >= 0x40 {
            if data.len() < 2 {
                return (0, 0);
            }
            let raw = data[1] as u32;
            return (((((p & 0x3F) as u32) << 8) | raw).wrapping_add(128), 2);
        }

        // len=3: prefix >= 0x20 (001x_xxxx)
        if p >= 0x20 {
            if data.len() < 3 {
                return (0, 0);
            }
            let raw = u16::from_le_bytes([data[1], data[2]]) as u32;
            return (((((p & 0x1F) as u32) << 16) | raw).wrapping_add(16512), 3);
        }

        // len=4: prefix >= 0x10 (0001_xxxx)
        if p >= 0x10 {
            if data.len() < 4 {
                return (0, 0);
            }
            let raw = u32::from_le_bytes([data[1], data[2], data[3], 0]) & 0xFF_FFFF;
            return (((((p & 0x0F) as u32) << 24) | raw).wrapping_add(2113664), 4);
        }

        // len=5: prefix >= 0x08 (0000_1xxx)
        if data.len() < 5 {
            return (0, 0);
        }
        let raw = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
        (
            ((((p & 0x07) as u64) << 32) | raw as u64).wrapping_add(270549120) as u32,
            5,
        )
    }
}

/// An unsigned 32-bit integer in variable-length quantity encoding.
///
/// Stored as a byte array containing the encoded representation.
#[derive(Clone, Copy)]
pub struct Vu32(pub(crate) [u8; VU32_BUF_SIZE]);

#[allow(clippy::len_without_is_empty)]
impl Vu32 {
    /// Construct a new VLQ instance from the given `u32`.
    #[inline(always)]
    pub fn new(value: u32) -> Vu32 {
        encode_vu32(value)
    }

    /// Retrieve the stored number as `u32`.
    #[inline(always)]
    pub fn get(&self) -> u32 {
        decode_vu32_slice(self.bytes()).0
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu32(self.0[0])
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub fn bytes(&self) -> &[u8] {
        &self.0[..self.len() as usize]
    }
}

impl From<u32> for Vu32 {
    fn from(n: u32) -> Self {
        encode_vu32(n)
    }
}

impl From<Vu32> for u32 {
    fn from(n: Vu32) -> Self {
        n.get()
    }
}

impl Display for Vu32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&u32::from(*self), f)
    }
}

impl Debug for Vu32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vu32(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
