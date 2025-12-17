//! Unsigned 64-bit VLQ encoding.

use core::fmt::{Debug, Display};

pub(crate) const VU64_BUF_SIZE: usize = 9;

/// Decode length from first byte for u64 (max 9 bytes).
#[inline(always)]
pub(crate) const fn decode_len_vu64(n: u8) -> u8 {
    n.leading_zeros() as u8 + 1
}

// Offset constants for ASM encoding (thresholds for length determination)
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const ENC_OFF2: u64 = 0x80;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const ENC_OFF3: u64 = 0x4080;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const ENC_OFF4: u64 = 0x20_4080;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const ENC_OFF5: u64 = 0x1020_4080;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const ENC_OFF6: u64 = 0x0008_1020_4080;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const ENC_OFF7: u64 = 0x0408_1020_4080;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const ENC_OFF8: u64 = 0x0002_0408_1020_4080;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const ENC_OFF9: u64 = 0x0102_0408_1020_4080;

// Masks for ASM data extraction (40-bit and 48-bit masks that can't be immediates)
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const MASK_40: u64 = 0xFF_FFFF_FFFF;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const MASK_48: u64 = 0xFFFF_FFFF_FFFF;
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
const MASK_56: u64 = 0xFF_FFFF_FFFF_FFFF;

// x86_64 offset constants for ASM encoding (thresholds for length determination)
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_OFF2: u64 = 0x80;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_OFF3: u64 = 0x4080;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_OFF4: u64 = 0x20_4080;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_OFF5: u64 = 0x1020_4080;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_OFF6: u64 = 0x0008_1020_4080;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_OFF7: u64 = 0x0408_1020_4080;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_OFF8: u64 = 0x0002_0408_1020_4080;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_OFF9: u64 = 0x0102_0408_1020_4080;

// x86_64 masks for ASM data extraction
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_MASK_40: u64 = 0xFF_FFFF_FFFF;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_MASK_48: u64 = 0xFFFF_FFFF_FFFF;
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
const X86_MASK_56: u64 = 0xFF_FFFF_FFFF_FFFF;

/// Encode a u64 in VLQ format using aarch64 inline asm.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn encode_vu64_asm(n: u64) -> (u8, u64) {
    let prefix: u64;
    let data: u64;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Compare against preloaded thresholds and branch
            "cmp    {val}, {off2}",
            "b.lo   100f",

            "cmp    {val}, {off3}",
            "b.lo   101f",

            "cmp    {val}, {off4}",
            "b.lo   102f",

            "cmp    {val}, {off5}",
            "b.lo   103f",

            "cmp    {val}, {off6}",
            "b.lo   104f",

            "cmp    {val}, {off7}",
            "b.lo   105f",

            "cmp    {val}, {off8}",
            "b.lo   106f",

            "cmp    {val}, {off9}",
            "b.lo   107f",

            // len=9: val >= offset!(9)
            "sub    {data}, {val}, {off9}",
            "mov    {prefix}, #0x00",
            "b      200f",

            // len=1: val < 128
            "100:",
            "orr    {prefix}, {val}, #0x80",
            "mov    {data}, #0",
            "b      200f",

            // len=2: offset = 128
            "101:",
            "sub    {data}, {val}, {off2}",
            "lsr    {prefix}, {data}, #8",
            "orr    {prefix}, {prefix}, #0x40",
            "and    {data}, {data}, #0xFF",
            "b      200f",

            // len=3: offset = 16512
            "102:",
            "sub    {data}, {val}, {off3}",
            "lsr    {prefix}, {data}, #16",
            "orr    {prefix}, {prefix}, #0x20",
            "and    {data}, {data}, #0xFFFF",
            "b      200f",

            // len=4: offset = 2113664
            "103:",
            "sub    {data}, {val}, {off4}",
            "lsr    {prefix}, {data}, #24",
            "orr    {prefix}, {prefix}, #0x10",
            "ubfx   {data}, {data}, #0, #24",
            "b      200f",

            // len=5: offset = 270549120
            "104:",
            "sub    {data}, {val}, {off5}",
            "lsr    {prefix}, {data}, #32",
            "orr    {prefix}, {prefix}, #0x08",
            "and    {data}, {data}, #0xFFFFFFFF",
            "b      200f",

            // len=6: offset = 34630287488
            "105:",
            "sub    {data}, {val}, {off6}",
            "lsr    {prefix}, {data}, #40",
            "orr    {prefix}, {prefix}, #0x04",
            "and    {data}, {data}, {mask40}",
            "b      200f",

            // len=7: offset = 4432676798592
            "106:",
            "sub    {data}, {val}, {off7}",
            "lsr    {prefix}, {data}, #48",
            "orr    {prefix}, {prefix}, #0x02",
            "and    {data}, {data}, {mask48}",
            "b      200f",

            // len=8: offset = 567382630219904
            "107:",
            "sub    {data}, {val}, {off8}",
            "mov    {prefix}, #0x01",
            "and    {data}, {data}, {mask56}",

            "200:",

            val = in(reg) n,
            off2 = in(reg) ENC_OFF2,
            off3 = in(reg) ENC_OFF3,
            off4 = in(reg) ENC_OFF4,
            off5 = in(reg) ENC_OFF5,
            off6 = in(reg) ENC_OFF6,
            off7 = in(reg) ENC_OFF7,
            off8 = in(reg) ENC_OFF8,
            off9 = in(reg) ENC_OFF9,
            mask40 = in(reg) MASK_40,
            mask48 = in(reg) MASK_48,
            mask56 = in(reg) MASK_56,
            data = out(reg) data,
            prefix = out(reg) prefix,
            options(pure, nomem, nostack),
        );
    }
    (prefix as u8, data)
}

/// Encode a u64 in VLQ format.
///
/// Returns Vu64(prefix, packed) where packed contains data bytes in LE order.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn encode_vu64(n: u64) -> Vu64 {
    let (prefix, data) = encode_vu64_asm(n);
    Vu64(prefix, data)
}

/// Encode a u64 in VLQ format using x86_64 inline asm.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn encode_vu64_asm_x86(n: u64) -> (u8, u64) {
    let prefix: u64;
    let data: u64;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Compare against preloaded thresholds and branch
            "cmp    {n:r}, {off2:r}",
            "jb     100f",

            "cmp    {n:r}, {off3:r}",
            "jb     101f",

            "cmp    {n:r}, {off4:r}",
            "jb     102f",

            "cmp    {n:r}, {off5:r}",
            "jb     103f",

            "cmp    {n:r}, {off6:r}",
            "jb     104f",

            "cmp    {n:r}, {off7:r}",
            "jb     105f",

            "cmp    {n:r}, {off8:r}",
            "jb     106f",

            "cmp    {n:r}, {off9:r}",
            "jb     107f",

            // len=9
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off9:r}",
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
            "sub    {data:r}, {off2:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 8",
            "or     {prefix:r}, 0x40",
            "and    {data:r}, 0xFF",
            "jmp    200f",

            // len=3
            "102:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off3:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 16",
            "or     {prefix:r}, 0x20",
            "and    {data:r}, 0xFFFF",
            "jmp    200f",

            // len=4
            "103:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off4:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 24",
            "or     {prefix:r}, 0x10",
            "and    {data:r}, 0xFFFFFF",
            "jmp    200f",

            // len=5
            "104:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off5:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 32",
            "or     {prefix:r}, 0x08",
            "and    {data:r}, 0xFFFFFFFF",
            "jmp    200f",

            // len=6
            "105:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off6:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 40",
            "or     {prefix:r}, 0x04",
            "and    {data:r}, {mask40:r}",
            "jmp    200f",

            // len=7
            "106:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off7:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 48",
            "or     {prefix:r}, 0x02",
            "and    {data:r}, {mask48:r}",
            "jmp    200f",

            // len=8
            "107:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off8:r}",
            "mov    {prefix:r}, 0x01",
            "and    {data:r}, {mask56:r}",

            "200:",

            n = in(reg) n,
            off2 = in(reg) X86_OFF2,
            off3 = in(reg) X86_OFF3,
            off4 = in(reg) X86_OFF4,
            off5 = in(reg) X86_OFF5,
            off6 = in(reg) X86_OFF6,
            off7 = in(reg) X86_OFF7,
            off8 = in(reg) X86_OFF8,
            off9 = in(reg) X86_OFF9,
            mask40 = in(reg) X86_MASK_40,
            mask48 = in(reg) X86_MASK_48,
            mask56 = in(reg) X86_MASK_56,
            prefix = out(reg) prefix,
            data = out(reg) data,
            options(pure, nomem, nostack),
        );
    }
    (prefix as u8, data)
}

/// Encode a u64 in VLQ format.
///
/// Returns Vu64(prefix, packed) where packed contains data bytes in LE order.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn encode_vu64(n: u64) -> Vu64 {
    let (prefix, data) = encode_vu64_asm_x86(n);
    Vu64(prefix, data)
}

/// Encode a u64 in VLQ format.
///
/// Returns Vu64(prefix, packed) where packed contains data bytes in LE order.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
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
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
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
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
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
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
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
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_vu64(n: Vu64) -> u64 {
    decode_vu64_asm(n.0, n.1)
}

/// Decode a little-endian VLQ using x86_64 inline asm with LZCNT.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
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
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn decode_vu64(n: Vu64) -> u64 {
    decode_vu64_asm_x86(n.0, n.1)
}

/// Decode a VLQ back to u64.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
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

// Byte masks for branchless decode: masks off unused bytes in 8-byte load
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
static BYTE_MASKS_64: [u64; 10] = [
    0,
    0,                  // len=1: 0 data bytes
    0xFF,               // len=2: 1 data byte
    0xFFFF,             // len=3: 2 data bytes
    0xFFFFFF,           // len=4: 3 data bytes
    0xFFFFFFFF,         // len=5: 4 data bytes
    0xFFFFFFFFFF,       // len=6: 5 data bytes
    0xFFFFFFFFFFFF,     // len=7: 6 data bytes
    0xFFFFFFFFFFFFFF,   // len=8: 7 data bytes
    0xFFFFFFFFFFFFFFFF, // len=9: 8 data bytes
];

/// Decode a u64 from a byte slice using CLZ dispatch aarch64 assembly.
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_vu64_slice(data: &[u8]) -> (u64, usize) {
    let data_len = data.len();

    if data_len == 0 {
        return (0, 0);
    }

    let value: u64;
    let len: usize;

    // Preload all offset constants
    const OFFSET2: u64 = 0x80;
    const OFFSET3: u64 = 0x4080;
    const OFFSET4: u64 = 0x20_4080;
    const OFFSET5: u64 = 0x1020_4080;
    const OFFSET6: u64 = 0x0008_1020_4080;
    const OFFSET7: u64 = 0x0408_1020_4080;
    const OFFSET8: u64 = 0x0002_0408_1020_4080;
    const OFFSET9: u64 = 0x0102_0408_1020_4080;

    // SAFETY: We've verified data is not empty. Bounds checked after decode.
    unsafe {
        core::arch::asm!(
            // Load prefix byte
            "ldrb   w3, [{ptr}]",

            // CLZ dispatch for all lengths (1-9)
            "clz    w4, w3",
            "sub    w4, w4, #24",              // len=1->0, ..., len=9->8

            // Computed branch: each handler is 32 bytes (8 instructions)
            "adr    x10, 1f",
            "add    x10, x10, x4, lsl #5",
            "br     x10",

            // len=1 handler
            ".p2align 5",
            "1:",
            "and    {out}, x3, #0x7F",
            "mov    {len:w}, #1",
            "b      100f",
            "nop", "nop", "nop", "nop", "nop",

            // len=2: offset=128 (preloaded)
            "ldrb   w5, [{ptr}, #1]",
            "add    x5, x5, {off2}",
            "and    x6, x3, #0x3F",
            "add    {out}, x5, x6, lsl #8",
            "mov    {len:w}, #2",
            "b      100f",
            "nop", "nop",

            // len=3: offset=0x4080 (preloaded)
            "ldrh   w5, [{ptr}, #1]",
            "add    x5, x5, {off3}",
            "and    x6, x3, #0x1F",
            "add    {out}, x5, x6, lsl #16",
            "mov    {len:w}, #3",
            "b      100f",
            "nop", "nop",

            // len=4: offset=0x204080 (uses preloaded constant)
            "ldr    w5, [{ptr}, #1]",
            "ubfx   x5, x5, #0, #24",
            "add    x5, x5, {off4}",
            "and    x6, x3, #0x0F",
            "add    {out}, x5, x6, lsl #24",
            "mov    {len:w}, #4",
            "b      100f",
            "nop",

            // len=5: offset=0x10204080 (preloaded)
            "ldr    w5, [{ptr}, #1]",
            "add    x5, x5, {off5}",
            "and    x6, x3, #0x07",
            "add    {out}, x5, x6, lsl #32",
            "mov    {len:w}, #5",
            "b      100f",
            "nop", "nop",

            // len=6: offset=0x0008_1020_4080 (preloaded)
            "ldr    x5, [{ptr}, #1]",
            "ubfx   x5, x5, #0, #40",
            "add    x5, x5, {off6}",
            "and    x6, x3, #0x03",
            "add    {out}, x5, x6, lsl #40",
            "mov    {len:w}, #6",
            "b      100f",
            "nop",

            // len=7: offset=0x0408_1020_4080 (preloaded)
            "ldr    x5, [{ptr}, #1]",
            "ubfx   x5, x5, #0, #48",
            "add    x5, x5, {off7}",
            "and    x6, x3, #0x01",
            "add    {out}, x5, x6, lsl #48",
            "mov    {len:w}, #7",
            "b      100f",
            "nop",

            // len=8: offset=0x0002_0408_1020_4080 (preloaded)
            "ldr    x5, [{ptr}, #1]",
            "ubfx   x5, x5, #0, #56",
            "add    {out}, x5, {off8}",
            "mov    {len:w}, #8",
            "b      100f",
            "nop", "nop", "nop", "nop",

            // len=9: offset=0x0102_0408_1020_4080 (preloaded)
            "ldr    x5, [{ptr}, #1]",
            "add    {out}, x5, {off9}",
            "mov    {len:w}, #9",
            "b      100f",
            "nop", "nop", "nop", "nop",

            "100:",

            ptr = in(reg) data.as_ptr(),
            off2 = in(reg) OFFSET2,
            off3 = in(reg) OFFSET3,
            off4 = in(reg) OFFSET4,
            off5 = in(reg) OFFSET5,
            off6 = in(reg) OFFSET6,
            off7 = in(reg) OFFSET7,
            off8 = in(reg) OFFSET8,
            off9 = in(reg) OFFSET9,
            out = out(reg) value,
            len = out(reg) len,
            out("w3") _, out("w4") _,
            out("x5") _, out("x6") _,
            out("x10") _,
            options(pure, readonly, nostack),
        );
    }

    // Bounds check after decode
    if len > data_len {
        return (0, 0);
    }

    (value, len)
}

/// Decode a u64 from a byte slice (fallback for non-aarch64 or no asm feature).
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
#[inline(always)]
pub fn decode_vu64_slice(data: &[u8]) -> (u64, usize) {
    let Some(&p) = data.first() else {
        return (0, 0);
    };

    // len=1: prefix >= 0x80 (1xxx_xxxx)
    if p >= 0x80 {
        return ((p & 0x7F) as u64, 1);
    }

    // len=2: prefix >= 0x40 (01xx_xxxx)
    if p >= 0x40 {
        if data.len() < 2 {
            return (0, 0);
        }
        let raw = data[1] as u64;
        return (((((p & 0x3F) as u64) << 8) | raw).wrapping_add(128), 2);
    }

    // len=3: prefix >= 0x20 (001x_xxxx)
    if p >= 0x20 {
        if data.len() < 3 {
            return (0, 0);
        }
        let raw = u16::from_le_bytes([data[1], data[2]]) as u64;
        return (((((p & 0x1F) as u64) << 16) | raw).wrapping_add(16512), 3);
    }

    // len=4: prefix >= 0x10 (0001_xxxx)
    if p >= 0x10 {
        if data.len() < 4 {
            return (0, 0);
        }
        let raw = u32::from_le_bytes([data[1], data[2], data[3], 0]) as u64 & 0xFF_FFFF;
        return (((((p & 0x0F) as u64) << 24) | raw).wrapping_add(2113664), 4);
    }

    // len=5: prefix >= 0x08 (0000_1xxx)
    if p >= 0x08 {
        if data.len() < 5 {
            return (0, 0);
        }
        let raw = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as u64;
        return (
            ((((p & 0x07) as u64) << 32) | raw).wrapping_add(270549120),
            5,
        );
    }

    // len=6: prefix >= 0x04 (0000_01xx)
    if p >= 0x04 {
        if data.len() < 6 {
            return (0, 0);
        }
        let raw = u64::from_le_bytes([data[1], data[2], data[3], data[4], data[5], 0, 0, 0])
            & 0xFF_FFFF_FFFF;
        return (
            ((((p & 0x03) as u64) << 40) | raw).wrapping_add(34630287488),
            6,
        );
    }

    // len=7: prefix >= 0x02 (0000_001x)
    if p >= 0x02 {
        if data.len() < 7 {
            return (0, 0);
        }
        let raw = u64::from_le_bytes([data[1], data[2], data[3], data[4], data[5], data[6], 0, 0])
            & 0xFFFF_FFFF_FFFF;
        return (
            ((((p & 0x01) as u64) << 48) | raw).wrapping_add(4432676798592),
            7,
        );
    }

    // len=8: prefix == 0x01 (0000_0001)
    if p == 0x01 {
        if data.len() < 8 {
            return (0, 0);
        }
        let raw = u64::from_le_bytes([
            data[1], data[2], data[3], data[4], data[5], data[6], data[7], 0,
        ]) & 0xFF_FFFF_FFFF_FFFF;
        return (raw.wrapping_add(567382630219904), 8);
    }

    // len=9: prefix == 0x00 (0000_0000)
    if data.len() < 9 {
        return (0, 0);
    }
    let raw = u64::from_le_bytes([
        data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
    ]);
    (raw.wrapping_add(72624976668147840), 9)
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
