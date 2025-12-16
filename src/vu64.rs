//! Unsigned 64-bit VLQ encoding.

use core::fmt::{Debug, Display};

pub(crate) const VU64_BUF_SIZE: usize = 9;

/// Decode length from first byte for u64 (max 9 bytes).
#[inline(always)]
pub(crate) const fn decode_len_vu64(n: u8) -> u8 {
    n.leading_zeros() as u8 + 1
}

/// Encode a u64 in VLQ format using aarch64 inline asm.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn encode_vu64_asm(n: u64) -> (u8, u64) {
    let prefix: u64;
    let data: u64;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Compare against thresholds and branch
            "cmp    x0, #128",
            "b.lo   100f",

            "mov    x4, #0x4080",
            "cmp    x0, x4",
            "b.lo   101f",

            "mov    x4, #0x4080",
            "movk   x4, #0x20, lsl #16",
            "cmp    x0, x4",
            "b.lo   102f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "cmp    x0, x4",
            "b.lo   103f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x8, lsl #32",
            "cmp    x0, x4",
            "b.lo   104f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x408, lsl #32",
            "cmp    x0, x4",
            "b.lo   105f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0002, lsl #48",
            "cmp    x0, x4",
            "b.lo   106f",

            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "cmp    x0, x4",
            "b.lo   107f",

            // len=9: offset = 72624976668147840 = 0x0102_0408_1020_4080
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "sub    x1, x0, x4",
            "mov    x2, #0x00",
            "b      200f",

            // len=1: n < 128
            "100:",
            "orr    x2, x0, #0x80",
            "mov    x1, #0",
            "b      200f",

            // len=2: offset = 128
            "101:",
            "sub    x1, x0, #128",
            "lsr    x2, x1, #8",
            "orr    x2, x2, #0x40",
            "and    x1, x1, #0xFF",
            "b      200f",

            // len=3: offset = 16512
            "102:",
            "mov    x4, #0x4080",
            "sub    x1, x0, x4",
            "lsr    x2, x1, #16",
            "orr    x2, x2, #0x20",
            "and    x1, x1, #0xFFFF",
            "b      200f",

            // len=4: offset = 2113664
            "103:",
            "mov    x4, #0x4080",
            "movk   x4, #0x20, lsl #16",
            "sub    x1, x0, x4",
            "lsr    x2, x1, #24",
            "orr    x2, x2, #0x10",
            "ubfx   x1, x1, #0, #24",
            "b      200f",

            // len=5: offset = 270549120
            "104:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "sub    x1, x0, x4",
            "lsr    x2, x1, #32",
            "orr    x2, x2, #0x08",
            "and    x1, x1, #0xFFFFFFFF",
            "b      200f",

            // len=6: offset = 34630287488
            "105:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x8, lsl #32",
            "sub    x1, x0, x4",
            "lsr    x2, x1, #40",
            "orr    x2, x2, #0x04",
            "mov    x5, #0xFFFFFFFFFF",
            "and    x1, x1, x5",
            "b      200f",

            // len=7: offset = 4432676798592
            "106:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x408, lsl #32",
            "sub    x1, x0, x4",
            "lsr    x2, x1, #48",
            "orr    x2, x2, #0x02",
            "mov    x5, #0xFFFFFFFFFFFF",
            "and    x1, x1, x5",
            "b      200f",

            // len=8: offset = 567382630219904
            "107:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0002, lsl #48",
            "sub    x1, x0, x4",
            "mov    x2, #0x01",
            "mov    x5, #0xFFFFFFFFFFFFFF",
            "and    x1, x1, x5",

            "200:",

            inout("x0") n => _,
            out("x1") data,
            out("x2") prefix,
            out("x4") _,
            out("x5") _,
            options(pure, nomem, nostack),
        );
    }
    (prefix as u8, data)
}

/// Encode a u64 in VLQ format.
///
/// Returns Vu64(prefix, packed) where packed contains data bytes in LE order.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn encode_vu64(n: u64) -> Vu64 {
    let (prefix, data) = encode_vu64_asm(n);
    Vu64(prefix, data)
}

/// Encode a u64 in VLQ format using x86_64 inline asm.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt"))]
#[inline(always)]
fn encode_vu64_asm_x86(n: u64) -> (u8, u64) {
    let prefix: u64;
    let data: u64;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Compare against thresholds and branch
            "cmp    {n:r}, 128",
            "jb     100f",

            "mov    {tmp:r}, 0x4080",
            "cmp    {n:r}, {tmp:r}",
            "jb     101f",

            "mov    {tmp:r}, 0x204080",
            "cmp    {n:r}, {tmp:r}",
            "jb     102f",

            "mov    {tmp:r}, 0x10204080",
            "cmp    {n:r}, {tmp:r}",
            "jb     103f",

            "movabs {tmp:r}, 0x0008_1020_4080",
            "cmp    {n:r}, {tmp:r}",
            "jb     104f",

            "movabs {tmp:r}, 0x0408_1020_4080",
            "cmp    {n:r}, {tmp:r}",
            "jb     105f",

            "movabs {tmp:r}, 0x0002_0408_1020_4080",
            "cmp    {n:r}, {tmp:r}",
            "jb     106f",

            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "cmp    {n:r}, {tmp:r}",
            "jb     107f",

            // len=9
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {tmp:r}",
            "xor    {prefix:r}, {prefix:r}",
            "jmp    200f",

            // len=1
            "100:",
            "mov    {prefix:r}, {n:r}",
            "or     {prefix:r}, 0x80",
            "xor    {data:r}, {data:r}",
            "jmp    200f",

            // len=2
            "101:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, 128",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 8",
            "or     {prefix:r}, 0x40",
            "and    {data:r}, 0xFF",
            "jmp    200f",

            // len=3
            "102:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, 0x4080",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 16",
            "or     {prefix:r}, 0x20",
            "and    {data:r}, 0xFFFF",
            "jmp    200f",

            // len=4
            "103:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, 0x204080",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 24",
            "or     {prefix:r}, 0x10",
            "and    {data:r}, 0xFFFFFF",
            "jmp    200f",

            // len=5
            "104:",
            "mov    {tmp:r}, 0x10204080",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {tmp:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 32",
            "or     {prefix:r}, 0x08",
            "mov    {tmp:e}, 0xFFFFFFFF",
            "and    {data:r}, {tmp:r}",
            "jmp    200f",

            // len=6
            "105:",
            "movabs {tmp:r}, 0x0008_1020_4080",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {tmp:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 40",
            "or     {prefix:r}, 0x04",
            "movabs {tmp:r}, 0xFFFFFFFFFF",
            "and    {data:r}, {tmp:r}",
            "jmp    200f",

            // len=7
            "106:",
            "movabs {tmp:r}, 0x0408_1020_4080",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {tmp:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 48",
            "or     {prefix:r}, 0x02",
            "movabs {tmp:r}, 0xFFFFFFFFFFFF",
            "and    {data:r}, {tmp:r}",
            "jmp    200f",

            // len=8
            "107:",
            "movabs {tmp:r}, 0x0002_0408_1020_4080",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {tmp:r}",
            "mov    {prefix:r}, 0x01",
            "movabs {tmp:r}, 0xFFFFFFFFFFFFFF",
            "and    {data:r}, {tmp:r}",

            "200:",

            n = in(reg) n,
            prefix = out(reg) prefix,
            data = out(reg) data,
            tmp = out(reg) _,
            options(pure, nomem, nostack),
        );
    }
    (prefix as u8, data)
}

/// Encode a u64 in VLQ format.
///
/// Returns Vu64(prefix, packed) where packed contains data bytes in LE order.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt"))]
#[inline(always)]
pub fn encode_vu64(n: u64) -> Vu64 {
    let (prefix, data) = encode_vu64_asm_x86(n);
    Vu64(prefix, data)
}

/// Encode a u64 in VLQ format.
///
/// Returns Vu64(prefix, packed) where packed contains data bytes in LE order.
#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "lzcnt")
)))]
#[inline(always)]
pub const fn encode_vu64(n: u64) -> Vu64 {
    if n < offset!(2) as u64 {
        // len=1: all data in prefix
        Vu64(0x80 | (n as u8), 0)
    } else if n < offset!(3) as u64 {
        // len=2: 1 data byte
        let val = n - offset!(2) as u64;
        Vu64(0x40 | ((val >> 8) as u8), val & 0xFF)
    } else if n < offset!(4) as u64 {
        // len=3: 2 data bytes
        let val = n - offset!(3) as u64;
        Vu64(0x20 | ((val >> 16) as u8), val & 0xFFFF)
    } else if n < offset!(5) {
        // len=4: 3 data bytes
        let val = n - offset!(4) as u64;
        Vu64(0x10 | ((val >> 24) as u8), val & 0xFF_FFFF)
    } else if n < offset!(6) {
        // len=5: 4 data bytes
        let val = n - offset!(5);
        Vu64(0x08 | ((val >> 32) as u8), val & 0xFFFF_FFFF)
    } else if n < offset!(7) {
        // len=6: 5 data bytes
        let val = n - offset!(6);
        Vu64(0x04 | ((val >> 40) as u8), val & 0xFF_FFFF_FFFF)
    } else if n < offset!(8) {
        // len=7: 6 data bytes
        let val = n - offset!(7);
        Vu64(0x02 | ((val >> 48) as u8), val & 0xFFFF_FFFF_FFFF)
    } else if n < offset!(9) {
        // len=8: 7 data bytes
        let val = n - offset!(8);
        Vu64(0x01, val & 0xFF_FFFF_FFFF_FFFF)
    } else {
        // len=9: 8 data bytes
        let val = n - offset!(9);
        Vu64(0x00, val)
    }
}

// Lookup tables for decode (fallback when no asm available)
#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "lzcnt")
)))]
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

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "lzcnt")
)))]
const MASKS_LE: [u8; 10] = [
    0, 0x7F, // len=1: 7 bits
    0x3F, // len=2: 6 bits
    0x1F, // len=3: 5 bits
    0x0F, // len=4: 4 bits
    0x07, // len=5: 3 bits
    0x03, // len=6: 2 bits
    0x01, // len=7: 1 bit
    0x00, // len=8: 0 bits
    0x00, // len=9: 0 bits
];

/// Decode a little-endian VLQ using branchless aarch64 inline asm.
/// No memory loads - all values computed or from jump table.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn decode_vu64_asm(prefix: u8, data: u64) -> u64 {
    let result: u64;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Get length
            "clz    w4, w3",
            "sub    w4, w4, #23",      // len = 1-9

            // Compute mask = 0xFF >> len (gives 0x7F for len=1, 0 for len>=8)
            "mov    w7, #0xFF",
            "lsr    w7, w7, w4",

            // Compute data_bits = (len - 1) * 8
            "sub    w8, w4, #1",
            "lsl    w8, w8, #3",

            // Jump table for offset (16-byte entries)
            "adr    x10, 100f",
            "sub    w11, w4, #1",
            "add    x10, x10, w11, uxtw #4",   // each entry is 16 bytes
            "br     x10",

            // Jump table entries (4 instructions = 16 bytes each)
            "100:",  // len=1: offset = 0
            "mov    x9, #0",
            "b      200f",
            "nop", "nop",

            // len=2: offset = 128
            "mov    x9, #128",
            "b      200f",
            "nop", "nop",

            // len=3: offset = 16512 = 0x4080
            "mov    x9, #0x4080",
            "b      200f",
            "nop", "nop",

            // len=4: offset = 2113664 = 0x204080
            "mov    x9, #0x4080",
            "movk   x9, #0x20, lsl #16",
            "b      200f",
            "nop",

            // len=5: offset = 270549120 = 0x10204080
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "b      200f",
            "nop",

            // len=6: offset = 34630287488 = 0x0008_1020_4080
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x8, lsl #32",
            "b      200f",

            // len=7: offset = 4432676798592 = 0x0408_1020_4080
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x408, lsl #32",
            "b      200f",

            // len=8: offset = 567382630219904 = 0x0002_0408_1020_4080 (needs 5 ops, branch to 300f)
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "b      300f",

            // len=9: offset = 72624976668147840 = 0x0102_0408_1020_4080 (falls through)
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",

            "200:",
            // result = ((prefix & mask) << data_bits) | data + offset
            "and    w5, w3, w7",
            "lsl    x5, x5, x8",
            "orr    x5, x5, x1",
            "add    x0, x5, x9",
            "b      400f",

            // len=8 finish (couldn't fit 5th instruction in 16-byte entry)
            "300:",
            "movk   x9, #0x0002, lsl #48",
            "b      200b",

            "400:",

            in("w3") prefix as u32,
            in("x1") data,
            out("x0") result,
            out("w4") _,
            out("x5") _,
            out("w7") _,
            out("w8") _,
            out("x9") _,
            out("x10") _,
            out("w11") _,
            options(pure, nomem, nostack),
        );
    }
    result
}

/// Decode a VLQ back to u64.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn decode_vu64(n: Vu64) -> u64 {
    decode_vu64_asm(n.0, n.1)
}

/// Decode a little-endian VLQ using x86_64 inline asm with LZCNT.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt"))]
#[inline(always)]
fn decode_vu64_asm_x86(prefix: u8, data: u64) -> u64 {
    let result: u64;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Get length: lzcnt on byte in 32-bit reg gives 24 + leading zeros
            "movzx  {prefix:e}, {prefix:l}",
            "lzcnt  {len:e}, {prefix:e}",
            "sub    {len:e}, 23",

            // Compute mask = 0xFF >> len
            "mov    {mask:e}, 0xFF",
            "mov    ecx, {len:e}",
            "shr    {mask:e}, cl",

            // Compute data_bits = (len - 1) * 8
            "mov    {shift:e}, {len:e}",
            "sub    {shift:e}, 1",
            "shl    {shift:e}, 3",

            // Jump table
            "lea    {jump:r}, [rip + 100f]",
            "mov    {idx:e}, {len:e}",
            "sub    {idx:e}, 1",
            "imul   {idx:e}, {idx:e}, 24",
            "add    {jump:r}, {idx:r}",
            "jmp    {jump:r}",

            // len=1: offset = 0
            "100:",
            "xor    {off:r}, {off:r}",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=2: offset = 128
            "mov    {off:r}, 128",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=3: offset = 16512
            "mov    {off:r}, 0x4080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=4: offset = 2113664
            "mov    {off:r}, 0x204080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=5: offset = 270549120
            "mov    {off:r}, 0x10204080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=6: offset = 34630287488
            "mov    {off:r}, 0x0008_1020_4080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=7: offset = 4432676798592
            "mov    {off:r}, 0x0408_1020_4080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=8: offset = 567382630219904
            "movabs {off:r}, 0x0002_0408_1020_4080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90",

            // len=9: offset = 72624976668147840
            "movabs {off:r}, 0x0102_0408_1020_4080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90",

            "200:",
            // result = ((prefix & mask) << data_bits) | data + offset
            "and    {prefix:e}, {mask:e}",
            "mov    ecx, {shift:e}",
            "shl    {prefix:r}, cl",
            "or     {prefix:r}, {data:r}",
            "add    {prefix:r}, {off:r}",

            prefix = inout(reg) prefix as u64 => result,
            data = in(reg) data,
            len = out(reg) _,
            mask = out(reg) _,
            shift = out(reg) _,
            jump = out(reg) _,
            idx = out(reg) _,
            off = out(reg) _,
            out("ecx") _,
            options(pure, nomem, nostack),
        );
    }
    result
}

/// Decode a VLQ back to u64.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt"))]
#[inline(always)]
pub fn decode_vu64(n: Vu64) -> u64 {
    decode_vu64_asm_x86(n.0, n.1)
}

/// Decode a VLQ back to u64.
#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "lzcnt")
)))]
#[inline(always)]
pub const fn decode_vu64(n: Vu64) -> u64 {
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

/// Decode a u64 from a byte slice.
///
/// Returns `Some((value, bytes_consumed))` on success, or `None` if the slice is too short.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn decode_vu64_slice(data: &[u8]) -> Option<(u64, usize)> {
    let first = *data.first()?;
    let len = decode_len_vu64(first) as usize;
    if data.len() < len {
        return None;
    }

    // Fused load + decode: branch on length to do minimal loads
    let ptr = data.as_ptr();
    let result: u64;
    unsafe {
        core::arch::asm!(
            // Jump table for load + decode (16-byte entries)
            "adr    x10, 100f",
            "sub    w11, w5, #1",
            "add    x10, x10, w11, uxtw #4",
            "br     x10",

            // len=1: all data in prefix, no load needed
            "100:",
            "and    x0, x3, #0x7F",
            "b      200f",
            "nop", "nop",

            // len=2: load 1 byte, offset=128
            "ldrb   w1, [x4, #1]",
            "and    w6, w3, #0x3F",
            "orr    x0, x1, x6, lsl #8",
            "b      201f",

            // len=3: load 2 bytes (ldrh), offset=16512
            "ldrh   w1, [x4, #1]",
            "and    w6, w3, #0x1F",
            "orr    x0, x1, x6, lsl #16",
            "b      202f",

            // len=4: load 3 bytes, offset=2113664
            "ldrh   w1, [x4, #1]",
            "ldrb   w6, [x4, #3]",
            "orr    w1, w1, w6, lsl #16",
            "b      203f",

            // len=5: load 4 bytes, offset=270549120
            "ldr    w1, [x4, #1]",
            "and    w6, w3, #0x07",
            "orr    x0, x1, x6, lsl #32",
            "b      204f",

            // len=6: load 5 bytes, offset=34630287488
            "ldr    w1, [x4, #1]",
            "ldrb   w6, [x4, #5]",
            "orr    x1, x1, x6, lsl #32",
            "b      205f",

            // len=7: load 6 bytes, offset=4432676798592
            "ldr    w1, [x4, #1]",
            "ldrh   w6, [x4, #5]",
            "orr    x1, x1, x6, lsl #32",
            "b      206f",

            // len=8: load 7 bytes, offset=567382630219904
            "ldr    w1, [x4, #1]",
            "ldrh   w6, [x4, #5]",
            "ldrb   w7, [x4, #7]",
            "b      207f",

            // len=9: load 8 bytes, offset=72624976668147840
            "ldr    x1, [x4, #1]",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "b      208f",

            // Finish paths with offset addition
            "200:",  // len=1 done (offset=0)
            "b      300f",

            "201:",  // len=2: add 128
            "add    x0, x0, #128",
            "b      300f",

            "202:",  // len=3: add 16512
            "mov    w9, #0x4080",
            "add    x0, x0, x9",
            "b      300f",

            "203:",  // len=4: finish load + add 2113664
            "and    w6, w3, #0x0F",
            "orr    x0, x1, x6, lsl #24",
            "mov    w9, #0x4080",
            "movk   w9, #0x20, lsl #16",
            "add    x0, x0, x9",
            "b      300f",

            "204:",  // len=5: add 270549120
            "mov    w9, #0x4080",
            "movk   w9, #0x1020, lsl #16",
            "add    x0, x0, x9",
            "b      300f",

            "205:",  // len=6: finish + add 34630287488
            "and    w6, w3, #0x03",
            "orr    x0, x1, x6, lsl #40",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x8, lsl #32",
            "add    x0, x0, x9",
            "b      300f",

            "206:",  // len=7: finish + add 4432676798592
            "and    w6, w3, #0x01",
            "orr    x0, x1, x6, lsl #48",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x408, lsl #32",
            "add    x0, x0, x9",
            "b      300f",

            "207:",  // len=8: finish load + add offset
            "orr    x1, x1, x6, lsl #32",
            "orr    x0, x1, x7, lsl #48",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0002, lsl #48",
            "add    x0, x0, x9",
            "b      300f",

            "208:",  // len=9: finish offset + add
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "add    x0, x1, x9",

            "300:",

            in("w3") first as u32,
            in("x4") ptr,
            in("w5") len as u32,
            out("x0") result,
            out("x1") _,
            out("w6") _,
            out("w7") _,
            out("x9") _,
            out("x10") _,
            out("w11") _,
            options(readonly, nostack),
        );
    }
    Some((result, len))
}

/// Decode a u64 from a byte slice.
///
/// Returns `Some((value, bytes_consumed))` on success, or `None` if the slice is too short.
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn decode_vu64_slice(data: &[u8]) -> Option<(u64, usize)> {
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
    Some((decode_vu64(Vu64(first, packed)), len))
}

/// An unsigned 64-bit integer in variable-length quantity encoding.
///
/// Stored as (prefix_byte, packed_data) to fit in two registers.
#[derive(Clone, Copy)]
pub struct Vu64(pub(crate) u8, pub(crate) u64);

#[allow(clippy::len_without_is_empty)]
impl Vu64 {
    /// Construct a new VLQ instance from the given `u64`.
    #[inline(always)]
    pub fn new(value: u64) -> Vu64 {
        encode_vu64(value)
    }

    /// Retrieve the stored number as `u64`.
    #[inline(always)]
    pub fn get(&self) -> u64 {
        decode_vu64(*self)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu64(self.0)
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
}

impl From<u64> for Vu64 {
    fn from(n: u64) -> Self {
        encode_vu64(n)
    }
}

impl From<Vu64> for u64 {
    fn from(n: Vu64) -> Self {
        decode_vu64(n)
    }
}

impl Display for Vu64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&u64::from(*self), f)
    }
}

impl Debug for Vu64 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vu64(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
