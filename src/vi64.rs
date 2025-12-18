//! Signed 64-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};

#[cfg(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
))]
use crate::vu64::VU64_BUF_SIZE;
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
use crate::vu64::encode_vu64;
use crate::vu64::{Vu64, decode_vu64_slice};

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

/// Fused zigzag + encode for i64 using aarch64 inline asm.
/// Writes directly to output buffer.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn encode_vi64_impl(n: i64, out: &mut [u8; VU64_BUF_SIZE]) {
    // SAFETY: Writing to valid buffer.
    unsafe {
        core::arch::asm!(
            // Zigzag encode: ((n << 1) ^ (n >> 63))
            "lsl    x4, {n}, #1",
            "eor    {n}, x4, {n}, asr #63",

            // Now {n} contains zigzag-encoded unsigned value
            // Compare against thresholds and branch
            "cmp    {n}, #128",
            "b.lo   100f",

            "mov    x4, #0x4080",
            "cmp    {n}, x4",
            "b.lo   101f",

            "mov    x4, #0x4080",
            "movk   x4, #0x20, lsl #16",
            "cmp    {n}, x4",
            "b.lo   102f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "cmp    {n}, x4",
            "b.lo   103f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x8, lsl #32",
            "cmp    {n}, x4",
            "b.lo   104f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x408, lsl #32",
            "cmp    {n}, x4",
            "b.lo   105f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0002, lsl #48",
            "cmp    {n}, x4",
            "b.lo   106f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "cmp    {n}, x4",
            "b.lo   107f",

            // len=9: offset = 72624976668147840 = 0x0102_0408_1020_4080
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "sub    {data}, {n}, x4",
            "mov    {prefix}, #0x00",
            "b      200f",

            // len=1: n < 128
            "100:",
            "orr    {prefix}, {n}, #0x80",
            "mov    {data}, #0",
            "b      200f",

            // len=2: offset = 128
            "101:",
            "sub    {data}, {n}, #128",
            "lsr    {prefix}, {data}, #8",
            "orr    {prefix}, {prefix}, #0x40",
            "b      200f",

            // len=3: offset = 16512
            "102:",
            "mov    x4, #0x4080",
            "sub    {data}, {n}, x4",
            "lsr    {prefix}, {data}, #16",
            "orr    {prefix}, {prefix}, #0x20",
            "b      200f",

            // len=4: offset = 2113664
            "103:",
            "mov    x4, #0x4080",
            "movk   x4, #0x20, lsl #16",
            "sub    {data}, {n}, x4",
            "lsr    {prefix}, {data}, #24",
            "orr    {prefix}, {prefix}, #0x10",
            "b      200f",

            // len=5: offset = 270549120
            "104:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "sub    {data}, {n}, x4",
            "lsr    {prefix}, {data}, #32",
            "orr    {prefix}, {prefix}, #0x08",
            "b      200f",

            // len=6: offset = 34630287488
            "105:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x8, lsl #32",
            "sub    {data}, {n}, x4",
            "lsr    {prefix}, {data}, #40",
            "orr    {prefix}, {prefix}, #0x04",
            "b      200f",

            // len=7: offset = 4432676798592
            "106:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x408, lsl #32",
            "sub    {data}, {n}, x4",
            "lsr    {prefix}, {data}, #48",
            "orr    {prefix}, {prefix}, #0x02",
            "b      200f",

            // len=8: offset = 567382630219904
            "107:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0002, lsl #48",
            "sub    {data}, {n}, x4",
            "mov    {prefix}, #0x01",

            "200:",
            // Write prefix byte and data to output buffer
            "strb   {prefix:w}, [{out}]",
            "str    {data}, [{out}, #1]",

            n = inout(reg) n => _,
            out = in(reg) out.as_mut_ptr(),
            data = out(reg) _,
            prefix = out(reg) _,
            out("x4") _,
            options(nostack),
        );
    }
}

/// Fused zigzag + encode for i64 using x86_64 inline asm.
/// Writes directly to output buffer.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn encode_vi64_impl(n: i64, out: &mut [u8; VU64_BUF_SIZE]) {
    // SAFETY: Writing to valid buffer.
    unsafe {
        core::arch::asm!(
            // Zigzag encode: ((n << 1) ^ (n >> 63))
            "mov    {zz:r}, {n:r}",
            "shl    {zz:r}, 1",
            "mov    {sign:r}, {n:r}",
            "sar    {sign:r}, 63",
            "xor    {zz:r}, {sign:r}",

            // Now {zz} contains zigzag-encoded unsigned value
            // Compare against thresholds and branch
            "cmp    {zz:r}, 128",
            "jb     100f",

            "mov    {tmp:r}, 0x4080",
            "cmp    {zz:r}, {tmp:r}",
            "jb     101f",

            "mov    {tmp:r}, 0x204080",
            "cmp    {zz:r}, {tmp:r}",
            "jb     102f",

            "mov    {tmp:r}, 0x10204080",
            "cmp    {zz:r}, {tmp:r}",
            "jb     103f",

            "movabs {tmp:r}, 0x0008_1020_4080",
            "cmp    {zz:r}, {tmp:r}",
            "jb     104f",

            "movabs {tmp:r}, 0x0408_1020_4080",
            "cmp    {zz:r}, {tmp:r}",
            "jb     105f",

            "movabs {tmp:r}, 0x0002_0408_1020_4080",
            "cmp    {zz:r}, {tmp:r}",
            "jb     106f",

            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "cmp    {zz:r}, {tmp:r}",
            "jb     107f",

            // len=9
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data:r}, {zz:r}",
            "sub    {data:r}, {tmp:r}",
            "xor    {prefix:r}, {prefix:r}",
            "jmp    200f",

            // len=1
            "100:",
            "mov    {prefix:r}, {zz:r}",
            "or     {prefix:r}, 0x80",
            "xor    {data:r}, {data:r}",
            "jmp    200f",

            // len=2
            "101:",
            "mov    {data:r}, {zz:r}",
            "sub    {data:r}, 128",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 8",
            "or     {prefix:r}, 0x40",
            "jmp    200f",

            // len=3
            "102:",
            "mov    {data:r}, {zz:r}",
            "sub    {data:r}, 0x4080",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 16",
            "or     {prefix:r}, 0x20",
            "jmp    200f",

            // len=4
            "103:",
            "mov    {data:r}, {zz:r}",
            "sub    {data:r}, 0x204080",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 24",
            "or     {prefix:r}, 0x10",
            "jmp    200f",

            // len=5
            "104:",
            "mov    {tmp:r}, 0x10204080",
            "mov    {data:r}, {zz:r}",
            "sub    {data:r}, {tmp:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 32",
            "or     {prefix:r}, 0x08",
            "jmp    200f",

            // len=6
            "105:",
            "movabs {tmp:r}, 0x0008_1020_4080",
            "mov    {data:r}, {zz:r}",
            "sub    {data:r}, {tmp:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 40",
            "or     {prefix:r}, 0x04",
            "jmp    200f",

            // len=7
            "106:",
            "movabs {tmp:r}, 0x0408_1020_4080",
            "mov    {data:r}, {zz:r}",
            "sub    {data:r}, {tmp:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 48",
            "or     {prefix:r}, 0x02",
            "jmp    200f",

            // len=8
            "107:",
            "movabs {tmp:r}, 0x0002_0408_1020_4080",
            "mov    {data:r}, {zz:r}",
            "sub    {data:r}, {tmp:r}",
            "mov    {prefix:r}, 0x01",

            "200:",
            // Write prefix byte and data to output buffer
            "mov    byte ptr [{out}], {prefix:l}",
            "mov    qword ptr [{out} + 1], {data:r}",

            n = in(reg) n,
            out = in(reg) out.as_mut_ptr(),
            zz = out(reg) _,
            sign = out(reg) _,
            prefix = out(reg) _,
            data = out(reg) _,
            tmp = out(reg) _,
            options(nostack),
        );
    }
}

/// Encode a signed i64 using zigzag encoding to VLQ.
#[inline(always)]
pub fn encode_vi64(n: i64) -> Vi64 {
    #[cfg(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
    ))]
    {
        let mut bytes = core::mem::MaybeUninit::<[u8; VU64_BUF_SIZE]>::uninit();
        // SAFETY: ASM writes 1 byte at offset 0 (prefix) and 8 bytes at offset 1 (data)
        unsafe {
            encode_vi64_impl(n, &mut *bytes.as_mut_ptr());
            Vi64(Vu64(bytes.assume_init()))
        }
    }

    #[cfg(not(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
    )))]
    {
        Vi64(encode_vu64(zigzag_encode_i64(n)))
    }
}

/// Decode a Vi64 back to a native i64.
#[inline(always)]
pub fn decode_vi64(n: Vi64) -> i64 {
    n.get()
}

/// Decode a Vi64 from a byte slice.
///
/// Returns (value, bytes_consumed) on success, or (0, 0) if the slice is empty/invalid.
#[inline(always)]
pub fn decode_vi64_slice(data: &[u8]) -> (i64, usize) {
    let (unsigned, len) = decode_vu64_slice(data);
    if len == 0 {
        return (0, 0);
    }
    (zigzag_decode_i64(unsigned), len)
}

/// A signed 64-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi64(Vu64);

#[allow(clippy::len_without_is_empty)]
impl Vi64 {
    /// Construct a new VLQ instance from the given `i64`.
    #[inline(always)]
    pub fn new(value: i64) -> Vi64 {
        encode_vi64(value)
    }

    /// Retrieve the stored number as `i64`.
    #[inline(always)]
    pub fn get(&self) -> i64 {
        zigzag_decode_i64(self.0.get())
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.0.len()
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub fn bytes(&self) -> &[u8] {
        self.0.bytes()
    }
}

impl From<i64> for Vi64 {
    fn from(n: i64) -> Self {
        encode_vi64(n)
    }
}

impl From<Vi64> for i64 {
    fn from(n: Vi64) -> Self {
        decode_vi64(n)
    }
}

impl Display for Vi64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&i64::from(*self), f)
    }
}

impl Debug for Vi64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vi64(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
