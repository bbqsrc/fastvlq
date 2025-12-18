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

/// Encode a u64 in VLQ format using aarch64 inline asm.
/// Writes directly to output buffer.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn encode_vu64_asm(n: u64, out: &mut [u8; VU64_BUF_SIZE]) {
    // SAFETY: Writing to valid buffer.
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
            "b      200f",

            // len=3: offset = 16512
            "102:",
            "sub    {data}, {val}, {off3}",
            "lsr    {prefix}, {data}, #16",
            "orr    {prefix}, {prefix}, #0x20",
            "b      200f",

            // len=4: offset = 2113664
            "103:",
            "sub    {data}, {val}, {off4}",
            "lsr    {prefix}, {data}, #24",
            "orr    {prefix}, {prefix}, #0x10",
            "b      200f",

            // len=5: offset = 270549120
            "104:",
            "sub    {data}, {val}, {off5}",
            "lsr    {prefix}, {data}, #32",
            "orr    {prefix}, {prefix}, #0x08",
            "b      200f",

            // len=6: offset = 34630287488
            "105:",
            "sub    {data}, {val}, {off6}",
            "lsr    {prefix}, {data}, #40",
            "orr    {prefix}, {prefix}, #0x04",
            "b      200f",

            // len=7: offset = 4432676798592
            "106:",
            "sub    {data}, {val}, {off7}",
            "lsr    {prefix}, {data}, #48",
            "orr    {prefix}, {prefix}, #0x02",
            "b      200f",

            // len=8: offset = 567382630219904
            "107:",
            "sub    {data}, {val}, {off8}",
            "mov    {prefix}, #0x01",

            "200:",
            // Write prefix byte and data to output buffer
            "strb   {prefix:w}, [{out}]",
            "str    {data}, [{out}, #1]",

            val = in(reg) n,
            out = in(reg) out.as_mut_ptr(),
            off2 = in(reg) ENC_OFF2,
            off3 = in(reg) ENC_OFF3,
            off4 = in(reg) ENC_OFF4,
            off5 = in(reg) ENC_OFF5,
            off6 = in(reg) ENC_OFF6,
            off7 = in(reg) ENC_OFF7,
            off8 = in(reg) ENC_OFF8,
            off9 = in(reg) ENC_OFF9,
            data = out(reg) _,
            prefix = out(reg) _,
            options(nostack),
        );
    }
}

/// Encode a u64 in VLQ format.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn encode_vu64(n: u64) -> Vu64 {
    let mut bytes = [0u8; VU64_BUF_SIZE];
    encode_vu64_asm(n, &mut bytes);
    Vu64(bytes)
}

/// Encode a u64 in VLQ format using x86_64 inline asm.
/// Writes directly to output buffer.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn encode_vu64_asm_x86(n: u64, out: &mut [u8; VU64_BUF_SIZE]) {
    // SAFETY: Writing to valid buffer.
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
            "jmp    200f",

            // len=3
            "102:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off3:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 16",
            "or     {prefix:r}, 0x20",
            "jmp    200f",

            // len=4
            "103:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off4:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 24",
            "or     {prefix:r}, 0x10",
            "jmp    200f",

            // len=5
            "104:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off5:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 32",
            "or     {prefix:r}, 0x08",
            "jmp    200f",

            // len=6
            "105:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off6:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 40",
            "or     {prefix:r}, 0x04",
            "jmp    200f",

            // len=7
            "106:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off7:r}",
            "mov    {prefix:r}, {data:r}",
            "shr    {prefix:r}, 48",
            "or     {prefix:r}, 0x02",
            "jmp    200f",

            // len=8
            "107:",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {off8:r}",
            "mov    {prefix:r}, 0x01",

            "200:",
            // Write prefix byte and data to output buffer
            "mov    byte ptr [{out}], {prefix:l}",
            "mov    qword ptr [{out} + 1], {data:r}",

            n = in(reg) n,
            out = in(reg) out.as_mut_ptr(),
            off2 = in(reg) X86_OFF2,
            off3 = in(reg) X86_OFF3,
            off4 = in(reg) X86_OFF4,
            off5 = in(reg) X86_OFF5,
            off6 = in(reg) X86_OFF6,
            off7 = in(reg) X86_OFF7,
            off8 = in(reg) X86_OFF8,
            off9 = in(reg) X86_OFF9,
            prefix = out(reg) _,
            data = out(reg) _,
            options(nostack),
        );
    }
}

/// Encode a u64 in VLQ format.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn encode_vu64(n: u64) -> Vu64 {
    let mut bytes = [0u8; VU64_BUF_SIZE];
    encode_vu64_asm_x86(n, &mut bytes);
    Vu64(bytes)
}

/// Encode a u64 in VLQ format.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
#[inline(always)]
pub const fn encode_vu64(n: u64) -> Vu64 {
    if n < offset!(2) as u64 {
        // len=1: all data in prefix
        Vu64([0x80 | (n as u8), 0, 0, 0, 0, 0, 0, 0, 0])
    } else if n < offset!(3) as u64 {
        // len=2: 1 data byte
        let val = n - offset!(2) as u64;
        Vu64([0x40 | ((val >> 8) as u8), val as u8, 0, 0, 0, 0, 0, 0, 0])
    } else if n < offset!(4) as u64 {
        // len=3: 2 data bytes
        let val = n - offset!(3) as u64;
        Vu64([
            0x20 | ((val >> 16) as u8),
            val as u8,
            (val >> 8) as u8,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    } else if n < offset!(5) {
        // len=4: 3 data bytes
        let val = n - offset!(4) as u64;
        Vu64([
            0x10 | ((val >> 24) as u8),
            val as u8,
            (val >> 8) as u8,
            (val >> 16) as u8,
            0,
            0,
            0,
            0,
            0,
        ])
    } else if n < offset!(6) {
        // len=5: 4 data bytes
        let val = n - offset!(5);
        Vu64([
            0x08 | ((val >> 32) as u8),
            val as u8,
            (val >> 8) as u8,
            (val >> 16) as u8,
            (val >> 24) as u8,
            0,
            0,
            0,
            0,
        ])
    } else if n < offset!(7) {
        // len=6: 5 data bytes
        let val = n - offset!(6);
        Vu64([
            0x04 | ((val >> 40) as u8),
            val as u8,
            (val >> 8) as u8,
            (val >> 16) as u8,
            (val >> 24) as u8,
            (val >> 32) as u8,
            0,
            0,
            0,
        ])
    } else if n < offset!(8) {
        // len=7: 6 data bytes
        let val = n - offset!(7);
        Vu64([
            0x02 | ((val >> 48) as u8),
            val as u8,
            (val >> 8) as u8,
            (val >> 16) as u8,
            (val >> 24) as u8,
            (val >> 32) as u8,
            (val >> 40) as u8,
            0,
            0,
        ])
    } else if n < offset!(9) {
        // len=8: 7 data bytes
        let val = n - offset!(8);
        Vu64([
            0x01,
            val as u8,
            (val >> 8) as u8,
            (val >> 16) as u8,
            (val >> 24) as u8,
            (val >> 32) as u8,
            (val >> 40) as u8,
            (val >> 48) as u8,
            0,
        ])
    } else {
        // len=9: 8 data bytes
        let val = n - offset!(9);
        Vu64([
            0x00,
            val as u8,
            (val >> 8) as u8,
            (val >> 16) as u8,
            (val >> 24) as u8,
            (val >> 32) as u8,
            (val >> 40) as u8,
            (val >> 48) as u8,
            (val >> 56) as u8,
        ])
    }
}

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

/// Decode a u64 from a byte slice using LZCNT dispatch x86_64 assembly.
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn decode_vu64_slice(data: &[u8]) -> (u64, usize) {
    let data_len = data.len();

    if data_len == 0 {
        return (0, 0);
    }

    let value: u64;
    let len: usize;

    // Offset constants
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
            // Load prefix byte and compute jump index
            "movzx  {prefix:e}, byte ptr [{ptr}]",
            "lzcnt  {idx:e}, {prefix:e}",
            "sub    {idx:e}, 24",              // idx = 0-8 for len 1-9
            "lea    {len:e}, [{idx:e} + 1]",   // len = idx + 1 = 1-9

            // Computed jump: 64-byte aligned handlers
            "shl    {idx:e}, 6",               // idx * 64
            "lea    {jump:r}, [rip + 1f]",
            "add    {jump:r}, {idx:r}",
            "jmp    {jump:r}",

            // Handler table - each handler padded to 64 bytes
            ".p2align 6",
            "1:",

            // len=1 (idx=0): value = prefix & 0x7F
            "and    {prefix:r}, 0x7F",
            "mov    {out:r}, {prefix:r}",
            "jmp    99f",
            ".p2align 6",

            // len=2 (idx=64): value = ((prefix & 0x3F) << 8) | data[1] + offset
            "movzx  {out:e}, byte ptr [{ptr} + 1]",
            "and    {prefix:e}, 0x3F",
            "shl    {prefix:r}, 8",
            "or     {out:r}, {prefix:r}",
            "add    {out:r}, {off2:r}",
            "jmp    99f",
            ".p2align 6",

            // len=3 (idx=128): 2 data bytes
            "movzx  {out:e}, word ptr [{ptr} + 1]",
            "and    {prefix:e}, 0x1F",
            "shl    {prefix:r}, 16",
            "or     {out:r}, {prefix:r}",
            "add    {out:r}, {off3:r}",
            "jmp    99f",
            ".p2align 6",

            // len=4 (idx=192): 3 data bytes
            "mov    {out:e}, dword ptr [{ptr} + 1]",
            "and    {out:r}, 0xFFFFFF",
            "and    {prefix:e}, 0x0F",
            "shl    {prefix:r}, 24",
            "or     {out:r}, {prefix:r}",
            "add    {out:r}, {off4:r}",
            "jmp    99f",
            ".p2align 6",

            // len=5 (idx=256): 4 data bytes
            "mov    {out:e}, dword ptr [{ptr} + 1]",
            "and    {prefix:e}, 0x07",
            "shl    {prefix:r}, 32",
            "or     {out:r}, {prefix:r}",
            "add    {out:r}, {off5:r}",
            "jmp    99f",
            ".p2align 6",

            // len=6 (idx=320): 5 data bytes (4 + 1 to avoid movabs mask)
            "mov    {out:e}, dword ptr [{ptr} + 1]",
            "movzx  {tmp:e}, byte ptr [{ptr} + 5]",
            "shl    {tmp:r}, 32",
            "or     {out:r}, {tmp:r}",
            "and    {prefix:e}, 0x03",
            "shl    {prefix:r}, 40",
            "or     {out:r}, {prefix:r}",
            "add    {out:r}, {off6:r}",
            "jmp    99f",
            ".p2align 6",

            // len=7 (idx=384): 6 data bytes (4 + 2 to avoid movabs mask)
            "mov    {out:e}, dword ptr [{ptr} + 1]",
            "movzx  {tmp:e}, word ptr [{ptr} + 5]",
            "shl    {tmp:r}, 32",
            "or     {out:r}, {tmp:r}",
            "and    {prefix:e}, 0x01",
            "shl    {prefix:r}, 48",
            "or     {out:r}, {prefix:r}",
            "add    {out:r}, {off7:r}",
            "jmp    99f",
            ".p2align 6",

            // len=8 (idx=448): 7 data bytes (4 + 3 to avoid movabs mask)
            "mov    {out:e}, dword ptr [{ptr} + 1]",
            "mov    {tmp:e}, dword ptr [{ptr} + 5]",
            "and    {tmp:r}, 0xFFFFFF",
            "shl    {tmp:r}, 32",
            "or     {out:r}, {tmp:r}",
            "add    {out:r}, {off8:r}",
            "jmp    99f",
            ".p2align 6",

            // len=9 (idx=512): 8 data bytes
            "mov    {out:r}, qword ptr [{ptr} + 1]",
            "add    {out:r}, {off9:r}",

            "99:",

            ptr = in(reg) data.as_ptr(),
            off2 = in(reg) OFFSET2,
            off3 = in(reg) OFFSET3,
            off4 = in(reg) OFFSET4,
            off5 = in(reg) OFFSET5,
            off6 = in(reg) OFFSET6,
            off7 = in(reg) OFFSET7,
            off8 = in(reg) OFFSET8,
            off9 = in(reg) OFFSET9,
            prefix = out(reg) _,
            out = out(reg) value,
            len = out(reg) len,
            tmp = out(reg) _,
            jump = out(reg) _,
            idx = out(reg) _,
            options(pure, readonly, nostack),
        );
    }

    // Bounds check after decode
    if len > data_len {
        return (0, 0);
    }

    (value, len)
}

/// Decode a u64 from a byte slice (fallback for no asm feature).
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
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
#[derive(Clone, Copy)]
pub struct Vu64(pub(crate) [u8; VU64_BUF_SIZE]);

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
        decode_vu64_slice(self.bytes()).0
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu64(self.0[0])
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub fn bytes(&self) -> &[u8] {
        &self.0[..self.len() as usize]
    }
}

impl From<u64> for Vu64 {
    fn from(n: u64) -> Self {
        encode_vu64(n)
    }
}

impl From<Vu64> for u64 {
    fn from(n: Vu64) -> Self {
        n.get()
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
