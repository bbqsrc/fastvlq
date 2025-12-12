//! Unsigned 32-bit VLQ encoding.

use core::fmt::{Debug, Display};

pub(crate) const VU32_BUF_SIZE: usize = 5;

/// Decode length from first byte for u32 (max 5 bytes).
#[inline(always)]
pub(crate) const fn decode_len_vu32(n: u8) -> u8 {
    let len = n.leading_zeros() as u8 + 1;
    if len > 5 { 5 } else { len }
}

#[inline(always)]
const fn encode_len_vu32(n: u32) -> u8 {
    match n {
        n if n < offset!(2) as u32 => 1,
        n if n < offset!(3) as u32 => 2,
        n if n < offset!(4) as u32 => 3,
        n if (n as u64) < offset!(5) => 4,
        _ => 5,
    }
}

/// Encode a u32 in value-length quantity encoding.
#[inline(always)]
#[must_use]
pub const fn encode_vu32(n: u32) -> Vu32 {
    let len = encode_len_vu32(n);
    let mut out_buf = [0u8; VU32_BUF_SIZE];

    match len {
        1 => {
            out_buf[0] = prefix!(1, n as u8);
        }
        2 => {
            let buf = encode_offset!(2, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 2);
            out_buf[0] = prefix!(2, buf[0]);
        }
        3 => {
            let buf = encode_offset!(3, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 3);
            out_buf[0] = prefix!(3, buf[0]);
        }
        4 => {
            let buf = encode_offset!(4, n).to_be_bytes();
            copy_from_slice_offset!(source = buf, dest = out_buf, offset = 4);
            out_buf[0] = prefix!(4, buf[0]);
        }
        _ => {
            // 5 bytes: value >= offset!(5)
            let n64 = n as u64;
            let buf = ((n64 - offset!(5)) << (8 * 3)).to_be_bytes();
            out_buf[0] = prefix!(5, buf[0]);
            out_buf[1] = buf[1];
            out_buf[2] = buf[2];
            out_buf[3] = buf[3];
            out_buf[4] = buf[4];
        }
    };

    Vu32(out_buf)
}

/// Decode a Vu32 back to a native u32.
#[inline(always)]
pub const fn decode_vu32(n: Vu32) -> u32 {
    let len = n.len();
    let n = n.bytes();

    match len {
        1 => unprefix!(1, n[0] as u32),
        2 => u32::from_le_bytes([n[1], unprefix!(2, n[0]), 0, 0]) + offset!(2) as u32,
        3 => u32::from_le_bytes([n[2], n[1], unprefix!(3, n[0]), 0]) + offset!(3) as u32,
        4 => u32::from_le_bytes([n[3], n[2], n[1], unprefix!(4, n[0])]) + offset!(4) as u32,
        _ => {
            // 5 bytes
            let val = u64::from_le_bytes([n[4], n[3], n[2], n[1], unprefix!(5, n[0]), 0, 0, 0]);
            (val + offset!(5)) as u32
        }
    }
}

/// An unsigned 32-bit integer in value-length quantity encoding.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vu32(pub(crate) [u8; VU32_BUF_SIZE]);

#[allow(clippy::len_without_is_empty)]
impl Vu32 {
    /// Construct a new VLQ instance from the given `u32`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u32) -> Vu32 {
        encode_vu32(value)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu32(self.0[0])
    }

    /// Retrieve the stored number as `u32`.
    #[inline(always)]
    pub const fn get(&self) -> u32 {
        decode_vu32(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; 5] {
        self.0
    }

    /// Get the serialized representation of the VLQ as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.0[..(self.len() as usize)]
    }
}

impl From<u32> for Vu32 {
    fn from(n: u32) -> Self {
        encode_vu32(n)
    }
}

impl From<Vu32> for u32 {
    fn from(n: Vu32) -> Self {
        decode_vu32(n)
    }
}

impl Display for Vu32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl Debug for Vu32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        write!(f, "Vu32(0b")?;
        for x in self.0.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", self.0[len]))
    }
}
