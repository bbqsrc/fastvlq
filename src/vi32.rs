//! Signed 32-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};

#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
use crate::vu32::encode_vu32;
use crate::vu32::{Vu32, decode_vu32, decode_vu32_slice};

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

/// Fused zigzag + encode for i32 using aarch64 inline asm.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn encode_vi32_asm(n: i32) -> (u8, u32) {
    let prefix: u32;
    let data: u32;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Zigzag encode: ((n << 1) ^ (n >> 31))
            "lsl    w4, w0, #1",
            "eor    w0, w4, w0, asr #31",

            // Now w0 contains zigzag-encoded unsigned value
            // Compare against thresholds and branch
            "cmp    w0, #128",
            "b.lo   100f",

            "mov    w4, #0x4080",
            "cmp    w0, w4",
            "b.lo   101f",

            "mov    w4, #0x4080",
            "movk   w4, #0x20, lsl #16",
            "cmp    w0, w4",
            "b.lo   102f",

            "mov    w4, #0x4080",
            "movk   w4, #0x1020, lsl #16",
            "cmp    w0, w4",
            "b.lo   103f",

            // len=5: offset = 270549120 = 0x10204080
            "mov    w4, #0x4080",
            "movk   w4, #0x1020, lsl #16",
            "sub    w1, w0, w4",
            "mov    w2, #0x08",
            "b      200f",

            // len=1: n < 128
            "100:",
            "orr    w2, w0, #0x80",
            "mov    w1, #0",
            "b      200f",

            // len=2: offset = 128
            "101:",
            "sub    w1, w0, #128",
            "lsr    w2, w1, #8",
            "orr    w2, w2, #0x40",
            "and    w1, w1, #0xFF",
            "b      200f",

            // len=3: offset = 16512 = 0x4080
            "102:",
            "mov    w4, #0x4080",
            "sub    w1, w0, w4",
            "lsr    w2, w1, #16",
            "orr    w2, w2, #0x20",
            "and    w1, w1, #0xFFFF",
            "b      200f",

            // len=4: offset = 2113664 = 0x204080
            "103:",
            "mov    w4, #0x4080",
            "movk   w4, #0x20, lsl #16",
            "sub    w1, w0, w4",
            "lsr    w2, w1, #24",
            "orr    w2, w2, #0x10",
            "ubfx   w1, w1, #0, #24",

            "200:",

            inout("w0") n => _,
            out("w1") data,
            out("w2") prefix,
            out("w4") _,
            options(pure, nomem, nostack),
        );
    }
    (prefix as u8, data)
}

/// Encode a signed i32 using zigzag encoding to VLQ.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn encode_vi32(n: i32) -> Vi32 {
    let (prefix, data) = encode_vi32_asm(n);
    Vi32(Vu32(prefix, data))
}

/// Fused zigzag + encode for i32 using x86_64 inline asm.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn encode_vi32_asm_x86(n: i32) -> (u8, u32) {
    let prefix: u32;
    let data: u32;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Zigzag encode: ((n << 1) ^ (n >> 31))
            "mov    {zz:e}, {n:e}",
            "shl    {zz:e}, 1",
            "mov    {sign:e}, {n:e}",
            "sar    {sign:e}, 31",
            "xor    {zz:e}, {sign:e}",

            // Now {zz} contains zigzag-encoded unsigned value
            // Compare against thresholds and branch
            "cmp    {zz:e}, 128",
            "jb     100f",

            "cmp    {zz:e}, 0x4080",
            "jb     101f",

            "cmp    {zz:e}, 0x204080",
            "jb     102f",

            "cmp    {zz:e}, 0x10204080",
            "jb     103f",

            // len=5
            "mov    {data:e}, {zz:e}",
            "sub    {data:e}, 0x10204080",
            "mov    {prefix:e}, 0x08",
            "jmp    200f",

            // len=1
            "100:",
            "mov    {prefix:e}, {zz:e}",
            "or     {prefix:e}, 0x80",
            "xor    {data:e}, {data:e}",
            "jmp    200f",

            // len=2
            "101:",
            "mov    {data:e}, {zz:e}",
            "sub    {data:e}, 128",
            "mov    {prefix:e}, {data:e}",
            "shr    {prefix:e}, 8",
            "or     {prefix:e}, 0x40",
            "and    {data:e}, 0xFF",
            "jmp    200f",

            // len=3
            "102:",
            "mov    {data:e}, {zz:e}",
            "sub    {data:e}, 0x4080",
            "mov    {prefix:e}, {data:e}",
            "shr    {prefix:e}, 16",
            "or     {prefix:e}, 0x20",
            "and    {data:e}, 0xFFFF",
            "jmp    200f",

            // len=4
            "103:",
            "mov    {data:e}, {zz:e}",
            "sub    {data:e}, 0x204080",
            "mov    {prefix:e}, {data:e}",
            "shr    {prefix:e}, 24",
            "or     {prefix:e}, 0x10",
            "and    {data:e}, 0xFFFFFF",

            "200:",

            n = in(reg) n,
            zz = out(reg) _,
            sign = out(reg) _,
            prefix = out(reg) prefix,
            data = out(reg) data,
            options(pure, nomem, nostack),
        );
    }
    (prefix as u8, data)
}

/// Encode a signed i32 using zigzag encoding to VLQ.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn encode_vi32(n: i32) -> Vi32 {
    let (prefix, data) = encode_vi32_asm_x86(n);
    Vi32(Vu32(prefix, data))
}

/// Encode a signed i32 using zigzag encoding to VLQ.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
#[inline(always)]
pub fn encode_vi32(n: i32) -> Vi32 {
    Vi32(encode_vu32(zigzag_encode_i32(n)))
}

/// Decode a Vi32 back to a native i32.
#[inline(always)]
pub fn decode_vi32(n: Vi32) -> i32 {
    zigzag_decode_i32(decode_vu32(n.0))
}

/// Decode a Vi32 from a byte slice.
///
/// Returns (value, bytes_consumed) on success, or (0, 0) if the slice is empty/invalid.
#[inline(always)]
pub fn decode_vi32_slice(data: &[u8]) -> (i32, usize) {
    let (unsigned, len) = decode_vu32_slice(data);
    if len == 0 {
        return (0, 0);
    }
    (zigzag_decode_i32(unsigned), len)
}

/// A signed 32-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi32(Vu32);

#[allow(clippy::len_without_is_empty)]
impl Vi32 {
    /// Construct a new VLQ instance from the given `i32`.
    #[inline(always)]
    pub fn new(value: i32) -> Vi32 {
        encode_vi32(value)
    }

    /// Retrieve the stored number as `i32`.
    #[inline(always)]
    pub fn get(&self) -> i32 {
        decode_vi32(*self)
    }

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
        Display::fmt(&i32::from(*self), f)
    }
}

impl Debug for Vi32 {
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
