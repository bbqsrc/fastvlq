//! Unsigned 64-bit VLQ encoding.

use core::fmt::{Debug, Display};
use core::marker::PhantomData;

use crate::{BE, LE};

pub(crate) const VU64_BUF_SIZE: usize = 9;

/// Decode length from first byte for u64 (max 9 bytes).
#[inline(always)]
pub(crate) const fn decode_len_vu64(n: u8) -> u8 {
    n.leading_zeros() as u8 + 1
}

/// Encode a u64 in big-endian VLQ format.
///
/// Returns Vu64(prefix, packed) where packed contains data bytes left-aligned.
#[inline(always)]
#[must_use]
pub const fn encode_vu64_be(n: u64) -> Vu64<BE> {
    if n < offset!(2) as u64 {
        // len=1: all data in prefix
        Vu64(0x80 | (n as u8), 0, PhantomData)
    } else if n < offset!(3) as u64 {
        // len=2: 1 data byte
        let val = n - offset!(2) as u64;
        Vu64(0x40 | ((val >> 8) as u8), val << 56, PhantomData)
    } else if n < offset!(4) as u64 {
        // len=3: 2 data bytes
        let val = n - offset!(3) as u64;
        Vu64(0x20 | ((val >> 16) as u8), val << 48, PhantomData)
    } else if n < offset!(5) {
        // len=4: 3 data bytes
        let val = n - offset!(4) as u64;
        Vu64(0x10 | ((val >> 24) as u8), val << 40, PhantomData)
    } else if n < offset!(6) {
        // len=5: 4 data bytes
        let val = n - offset!(5);
        Vu64(0x08 | ((val >> 32) as u8), val << 32, PhantomData)
    } else if n < offset!(7) {
        // len=6: 5 data bytes
        let val = n - offset!(6);
        Vu64(0x04 | ((val >> 40) as u8), val << 24, PhantomData)
    } else if n < offset!(8) {
        // len=7: 6 data bytes
        let val = n - offset!(7);
        Vu64(0x02 | ((val >> 48) as u8), val << 16, PhantomData)
    } else if n < offset!(9) {
        // len=8: 7 data bytes
        let val = n - offset!(8);
        Vu64(0x01, val << 8, PhantomData)
    } else {
        // len=9: 8 data bytes
        let val = n - offset!(9);
        Vu64(0x00, val, PhantomData)
    }
}

/// Encode a u64 in little-endian VLQ format.
///
/// Returns Vu64(prefix, packed) where packed contains data bytes in LE order.
#[inline(always)]
#[must_use]
pub const fn encode_vu64_le(n: u64) -> Vu64<LE> {
    if n < offset!(2) as u64 {
        // len=1: all data in prefix
        Vu64(0x80 | (n as u8), 0, PhantomData)
    } else if n < offset!(3) as u64 {
        // len=2: 1 data byte
        let val = n - offset!(2) as u64;
        Vu64(0x40 | ((val >> 8) as u8), val & 0xFF, PhantomData)
    } else if n < offset!(4) as u64 {
        // len=3: 2 data bytes
        let val = n - offset!(3) as u64;
        Vu64(0x20 | ((val >> 16) as u8), val & 0xFFFF, PhantomData)
    } else if n < offset!(5) {
        // len=4: 3 data bytes
        let val = n - offset!(4) as u64;
        Vu64(0x10 | ((val >> 24) as u8), val & 0xFF_FFFF, PhantomData)
    } else if n < offset!(6) {
        // len=5: 4 data bytes
        let val = n - offset!(5);
        Vu64(0x08 | ((val >> 32) as u8), val & 0xFFFF_FFFF, PhantomData)
    } else if n < offset!(7) {
        // len=6: 5 data bytes
        let val = n - offset!(6);
        Vu64(0x04 | ((val >> 40) as u8), val & 0xFF_FFFF_FFFF, PhantomData)
    } else if n < offset!(8) {
        // len=7: 6 data bytes
        let val = n - offset!(7);
        Vu64(0x02 | ((val >> 48) as u8), val & 0xFFFF_FFFF_FFFF, PhantomData)
    } else if n < offset!(9) {
        // len=8: 7 data bytes
        let val = n - offset!(8);
        Vu64(0x01, val & 0xFF_FFFF_FFFF_FFFF, PhantomData)
    } else {
        // len=9: 8 data bytes
        let val = n - offset!(9);
        Vu64(0x00, val, PhantomData)
    }
}

// Lookup tables for decode (const for pure Rust)
const OFFSETS: [u64; 10] = [
    0,
    0,
    offset!(2) as u64,
    offset!(3) as u64,
    offset!(4) as u64,
    offset!(5),
    offset!(6),
    offset!(7),
    offset!(8),
    offset!(9),
];

const MASKS_BE: [u64; 10] = [
    0,
    0x7F,           // len=1: 7 bits from prefix
    0x3F_FF,        // len=2: 6+8 = 14 bits
    0x1F_FF_FF,     // len=3: 5+16 = 21 bits
    0x0F_FF_FF_FF,  // len=4: 4+24 = 28 bits
    0x07_FF_FF_FF_FF,        // len=5: 3+32 = 35 bits
    0x03_FF_FF_FF_FF_FF,     // len=6: 2+40 = 42 bits
    0x01_FF_FF_FF_FF_FF_FF,  // len=7: 1+48 = 49 bits
    0x00_FF_FF_FF_FF_FF_FF_FF,       // len=8: 0+56 = 56 bits
    0xFF_FF_FF_FF_FF_FF_FF_FF,       // len=9: 64 bits
];

// Shifts for BE: how much to shift the combined (prefix << 56 | data >> 8) right
const SHIFTS_BE: [u8; 10] = [
    0,
    56, // len=1
    48, // len=2
    40, // len=3
    32, // len=4
    24, // len=5
    16, // len=6
    8,  // len=7
    0,  // len=8
    0,  // len=9
];

// Masks for LE: how many bits from prefix contain data
const MASKS_LE: [u8; 10] = [
    0,
    0x7F, // len=1: 7 bits
    0x3F, // len=2: 6 bits
    0x1F, // len=3: 5 bits
    0x0F, // len=4: 4 bits
    0x07, // len=5: 3 bits
    0x03, // len=6: 2 bits
    0x01, // len=7: 1 bit
    0x00, // len=8: 0 bits
    0x00, // len=9: (handled separately)
];

// Data bit counts for LE: (len-1) * 8
const DATA_BITS_LE: [u8; 10] = [0, 0, 8, 16, 24, 32, 40, 48, 56, 64];

// Static versions for asm access
#[cfg(target_arch = "aarch64")]
static OFFSETS_STATIC: [u64; 10] = OFFSETS;
#[cfg(target_arch = "aarch64")]
static MASKS_BE_STATIC: [u64; 10] = MASKS_BE;
#[cfg(target_arch = "aarch64")]
static SHIFTS_BE_STATIC: [u8; 10] = SHIFTS_BE;
#[cfg(target_arch = "aarch64")]
static MASKS_LE_STATIC: [u8; 10] = MASKS_LE;
#[cfg(target_arch = "aarch64")]
static DATA_BITS_LE_STATIC: [u8; 10] = DATA_BITS_LE;

/// Decode a big-endian VLQ using aarch64 inline asm.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn decode_vu64_be_asm(prefix: u8, data: u64) -> u64 {
    let result: u64;
    // SAFETY: Pure computation, reads from static tables.
    unsafe {
        core::arch::asm!(
            // Get length: clz(prefix) + 1
            "clz    w4, w3",           // w4 = leading zeros of prefix (24 + actual)
            "sub    w4, w4, #23",      // w4 = len

            // Check for len == 1 (prefix >= 0x80)
            "cmp    w4, #1",
            "b.eq   2f",

            // Check for len == 9 (prefix == 0)
            "cmp    w4, #9",
            "b.eq   3f",

            // General case: len 2-8
            // combined = (prefix << 56) | (data >> 8)
            "lsl    x5, x3, #56",      // x5 = prefix << 56
            "lsr    x6, x1, #8",       // x6 = data >> 8
            "orr    x5, x5, x6",       // x5 = combined

            // Load shift, mask, offset from tables
            "ldrb   w7, [x10, w4, uxtw]", // w7 = shifts[len]
            "ldr    x8, [x11, x4, lsl #3]", // x8 = masks[len]
            "ldr    x9, [x12, x4, lsl #3]", // x9 = offsets[len]

            // result = ((combined >> shift) & mask) + offset
            "lsr    x5, x5, x7",
            "and    x5, x5, x8",
            "add    x0, x5, x9",
            "b      4f",

            // len == 1: result = prefix & 0x7F
            "2:",
            "and    x0, x3, #0x7F",
            "b      4f",

            // len == 9: result = data + OFFSETS[9]
            "3:",
            "ldr    x9, [x12, #72]",   // OFFSETS[9] at index 9 * 8 = 72
            "add    x0, x1, x9",

            "4:",
            in("w3") prefix as u32,
            in("x1") data,
            in("x10") SHIFTS_BE_STATIC.as_ptr(),
            in("x11") MASKS_BE_STATIC.as_ptr(),
            in("x12") OFFSETS_STATIC.as_ptr(),
            out("x0") result,
            out("w4") _,
            out("x5") _,
            out("x6") _,
            out("w7") _,
            out("x8") _,
            out("x9") _,
            options(pure, readonly, nostack),
        );
    }
    result
}

/// Decode a little-endian VLQ using aarch64 inline asm.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn decode_vu64_le_asm(prefix: u8, data: u64) -> u64 {
    let result: u64;
    // SAFETY: Pure computation, reads from static tables.
    unsafe {
        core::arch::asm!(
            // Get length: clz(prefix) + 1
            "clz    w4, w3",           // w4 = leading zeros
            "sub    w4, w4, #23",      // w4 = len

            // Check for len == 1
            "cmp    w4, #1",
            "b.eq   2f",

            // Check for len == 9
            "cmp    w4, #9",
            "b.eq   3f",

            // General case: len 2-8
            // prefix_bits = prefix & MASKS_LE[len]
            "ldrb   w7, [x10, w4, uxtw]", // w7 = masks_le[len]
            "and    w5, w3, w7",       // w5 = prefix & mask

            // data_bits from table
            "ldrb   w8, [x11, w4, uxtw]", // w8 = data_bits[len]

            // result = (prefix_bits << data_bits) | data + offset
            "lsl    x5, x5, x8",       // x5 = prefix_bits << data_bits
            "orr    x5, x5, x1",       // x5 |= data

            // Load offset
            "ldr    x9, [x12, x4, lsl #3]", // x9 = offsets[len]
            "add    x0, x5, x9",
            "b      4f",

            // len == 1: result = prefix & 0x7F
            "2:",
            "and    x0, x3, #0x7F",
            "b      4f",

            // len == 9: result = data + OFFSETS[9]
            "3:",
            "ldr    x9, [x12, #72]",
            "add    x0, x1, x9",

            "4:",
            in("w3") prefix as u32,
            in("x1") data,
            in("x10") MASKS_LE_STATIC.as_ptr(),
            in("x11") DATA_BITS_LE_STATIC.as_ptr(),
            in("x12") OFFSETS_STATIC.as_ptr(),
            out("x0") result,
            out("w4") _,
            out("x5") _,
            out("w7") _,
            out("x8") _,
            out("x9") _,
            options(pure, readonly, nostack),
        );
    }
    result
}

/// Decode a big-endian VLQ back to u64.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn decode_vu64_be(n: Vu64<BE>) -> u64 {
    decode_vu64_be_asm(n.0, n.1)
}

/// Decode a big-endian VLQ back to u64.
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub const fn decode_vu64_be(n: Vu64<BE>) -> u64 {
    let len = (n.0.leading_zeros() as usize) + 1;
    let prefix = n.0;
    let data = n.1;

    if len == 1 {
        (prefix & 0x7F) as u64
    } else if len == 9 {
        data + OFFSETS[9]
    } else {
        let prefix_bits = (prefix as u64) << 56;
        let combined = prefix_bits | (data >> 8);
        let shift = SHIFTS_BE[len];
        let mask = MASKS_BE[len];
        ((combined >> shift) & mask) + OFFSETS[len]
    }
}

/// Decode a little-endian VLQ back to u64.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn decode_vu64_le(n: Vu64<LE>) -> u64 {
    decode_vu64_le_asm(n.0, n.1)
}

/// Decode a little-endian VLQ back to u64.
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub const fn decode_vu64_le(n: Vu64<LE>) -> u64 {
    let len = (n.0.leading_zeros() as usize) + 1;
    let prefix = n.0;
    let data = n.1;

    if len == 1 {
        (prefix & 0x7F) as u64
    } else if len == 9 {
        data + OFFSETS[9]
    } else {
        let prefix_bits = (prefix & MASKS_LE[len]) as u64;
        let data_bits = (len - 1) * 8;
        ((prefix_bits << data_bits) | data) + OFFSETS[len]
    }
}

/// Decode a u64 from a big-endian byte slice.
///
/// Returns `Some((value, bytes_consumed))` on success, or `None` if the slice is too short.
#[inline(always)]
pub fn decode_vu64_slice_be(data: &[u8]) -> Option<(u64, usize)> {
    let first = *data.first()?;
    let len = decode_len_vu64(first) as usize;
    if data.len() < len {
        return None;
    }

    // Pack into (prefix, data_u64)
    let mut buf = [0u8; 8];
    if len > 1 {
        buf[..(len - 1)].copy_from_slice(&data[1..len]);
    }
    let packed = u64::from_be_bytes(buf);
    Some((decode_vu64_be(Vu64(first, packed, PhantomData)), len))
}

/// Decode a u64 from a little-endian byte slice.
///
/// Returns `Some((value, bytes_consumed))` on success, or `None` if the slice is too short.
#[inline(always)]
pub fn decode_vu64_slice_le(data: &[u8]) -> Option<(u64, usize)> {
    let first = *data.first()?;
    let len = decode_len_vu64(first) as usize;
    if data.len() < len {
        return None;
    }

    // Pack into (prefix, data_u64) with LE byte order
    let mut buf = [0u8; 8];
    if len > 1 {
        buf[..(len - 1)].copy_from_slice(&data[1..len]);
    }
    let packed = u64::from_le_bytes(buf);
    Some((decode_vu64_le(Vu64(first, packed, PhantomData)), len))
}

/// An unsigned 64-bit integer in variable-length quantity encoding.
///
/// Stored as (prefix_byte, packed_data) to fit in two registers.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vu64<E>(pub(crate) u8, pub(crate) u64, pub(crate) PhantomData<E>);

#[allow(clippy::len_without_is_empty)]
impl<E> Vu64<E> {
    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu64(self.0)
    }
}

impl Vu64<BE> {
    /// Construct a new big-endian VLQ instance from the given `u64`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u64) -> Vu64<BE> {
        encode_vu64_be(value)
    }

    /// Retrieve the stored number as `u64`.
    #[inline(always)]
    pub fn get(&self) -> u64 {
        decode_vu64_be(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; VU64_BUF_SIZE] {
        let mut out = [0u8; VU64_BUF_SIZE];
        out[0] = self.0;
        let len = self.len() as usize;
        if len > 1 {
            let data = self.1.to_be_bytes();
            let mut i = 0;
            while i < len - 1 {
                out[i + 1] = data[i];
                i += 1;
            }
        }
        out
    }

    /// Get the serialized representation of the VLQ as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        // We need to return a reference, but we only have (u8, u64)
        // This requires reconstructing bytes - for now return a static slice approach
        // Actually we can't return a slice to local data, so we need a different approach
        // Let's make this return the bytes array and let caller slice it
        unimplemented!("as_slice requires heap allocation or different design")
    }
}

impl Vu64<LE> {
    /// Construct a new little-endian VLQ instance from the given `u64`.
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u64) -> Vu64<LE> {
        encode_vu64_le(value)
    }

    /// Retrieve the stored number as `u64`.
    #[inline(always)]
    pub fn get(&self) -> u64 {
        decode_vu64_le(*self)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; VU64_BUF_SIZE] {
        let mut out = [0u8; VU64_BUF_SIZE];
        out[0] = self.0;
        let len = self.len() as usize;
        if len > 1 {
            let data = self.1.to_le_bytes();
            let mut i = 0;
            while i < len - 1 {
                out[i + 1] = data[i];
                i += 1;
            }
        }
        out
    }

    /// Get the serialized representation of the VLQ as a slice.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        unimplemented!("as_slice requires heap allocation or different design")
    }
}

impl From<u64> for Vu64<BE> {
    fn from(n: u64) -> Self {
        encode_vu64_be(n)
    }
}

impl From<u64> for Vu64<LE> {
    fn from(n: u64) -> Self {
        encode_vu64_le(n)
    }
}

impl From<Vu64<BE>> for u64 {
    fn from(n: Vu64<BE>) -> Self {
        decode_vu64_be(n)
    }
}

impl From<Vu64<LE>> for u64 {
    fn from(n: Vu64<LE>) -> Self {
        decode_vu64_le(n)
    }
}

impl<E> Display for Vu64<E>
where
    Vu64<E>: Copy,
    u64: From<Vu64<E>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&u64::from(*self), f)
    }
}

impl Vu64<BE> {
    fn fmt_debug(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vu64(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}

impl Vu64<LE> {
    fn fmt_debug(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vu64(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}

impl Debug for Vu64<BE> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.fmt_debug(f)
    }
}

impl Debug for Vu64<LE> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.fmt_debug(f)
    }
}
