//! Unsigned 64-bit VLQ encoding.

use core::fmt::{Debug, Display};

pub(crate) const VU64_BUF_SIZE: usize = 9;

/// Decode length from first byte for u64 (max 9 bytes).
#[inline(always)]
pub(crate) const fn decode_len_vu64(n: u8) -> u8 {
    n.leading_zeros() as u8 + 1
}

// Offset constants for ASM encoding (thresholds for length determination)
#[cfg(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(
        target_arch = "x86_64",
        target_feature = "lzcnt",
        target_feature = "bmi2",
        feature = "asm"
    )
))]
pub(crate) mod enc_offsets {
    pub const OFF2: u64 = 0x80;
    pub const OFF3: u64 = 0x4080;
    pub const OFF4: u64 = 0x20_4080;
    pub const OFF5: u64 = 0x1020_4080;
    pub const OFF6: u64 = 0x0008_1020_4080;
    pub const OFF7: u64 = 0x0408_1020_4080;
    pub const OFF8: u64 = 0x0002_0408_1020_4080;
    pub const OFF9: u64 = 0x0102_0408_1020_4080;
}

/// Encode a u64 in VLQ format using aarch64 inline asm with logarithm-based dispatch.
/// Uses the formula: idx = (56 - clz(127*n + 128)) / 7
/// This gives exact handler index without boundary checks in handlers.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn encode_vu64_impl(n: u64, out: &mut [u8; VU64_BUF_SIZE]) {
    use enc_offsets::*;
    // SAFETY: Writing to valid buffer.
    unsafe {
        core::arch::asm!(
            // Check for len=9 first (avoids overflow in 127*n + 128)
            "cmp    {val}, {off9}",
            "b.hs   9f",

            // Compute 127*n + 128 using madd: tmp = 127*val + 128
            "mov    {tmp}, #127",
            "mov    {idx}, #128",
            "madd   {tmp}, {val}, {tmp}, {idx}",

            // Get CLZ and compute idx = (56 - clz) / 7
            "clz    {tmp}, {tmp}",
            "mov    {idx}, #56",
            "sub    {idx}, {idx}, {tmp}",
            "mov    {tmp}, #37",
            "mul    {idx}, {idx}, {tmp}",
            "lsr    {idx}, {idx}, #8",

            // Computed branch (each handler is 16 bytes = 4 instructions)
            "adr    {jump}, 1f",
            "add    {jump}, {jump}, {idx}, lsl #4",
            "br     {jump}",

            // Handler table - each handler is exactly 16 bytes (4 instructions)
            ".p2align 4",
            "1:",

            // idx=0 (len=1): n < 128
            "orr    {prefix}, {val}, #0x80",
            "mov    {data}, #0",
            "b      200f",
            "nop",

            // idx=1 (len=2): 128 <= n < 16512
            "sub    {data}, {val}, {off2}",
            "lsr    {prefix}, {data}, #8",
            "orr    {prefix}, {prefix}, #0x40",
            "b      200f",

            // idx=2 (len=3): 16512 <= n < 2113664
            "sub    {data}, {val}, {off3}",
            "lsr    {prefix}, {data}, #16",
            "orr    {prefix}, {prefix}, #0x20",
            "b      200f",

            // idx=3 (len=4): 2113664 <= n < 270549120
            "sub    {data}, {val}, {off4}",
            "lsr    {prefix}, {data}, #24",
            "orr    {prefix}, {prefix}, #0x10",
            "b      200f",

            // idx=4 (len=5): 270549120 <= n < 34630287488
            "sub    {data}, {val}, {off5}",
            "lsr    {prefix}, {data}, #32",
            "orr    {prefix}, {prefix}, #0x08",
            "b      200f",

            // idx=5 (len=6): 34630287488 <= n < 4432676798592
            "sub    {data}, {val}, {off6}",
            "lsr    {prefix}, {data}, #40",
            "orr    {prefix}, {prefix}, #0x04",
            "b      200f",

            // idx=6 (len=7): 4432676798592 <= n < 567382630219904
            "sub    {data}, {val}, {off7}",
            "lsr    {prefix}, {data}, #48",
            "orr    {prefix}, {prefix}, #0x02",
            "b      200f",

            // idx=7 (len=8): 567382630219904 <= n < 72624976668147840
            "sub    {data}, {val}, {off8}",
            "mov    {prefix}, #0x01",
            "b      200f",
            "nop",

            // idx=8 (len=9): n >= 72624976668147840
            "9:",
            "sub    {data}, {val}, {off9}",
            "mov    {prefix}, #0x00",

            "200:",
            // Write prefix byte and data to output buffer
            "strb   {prefix:w}, [{out}]",
            "str    {data}, [{out}, #1]",

            val = in(reg) n,
            out = in(reg) out.as_mut_ptr(),
            off2 = in(reg) OFF2,
            off3 = in(reg) OFF3,
            off4 = in(reg) OFF4,
            off5 = in(reg) OFF5,
            off6 = in(reg) OFF6,
            off7 = in(reg) OFF7,
            off8 = in(reg) OFF8,
            off9 = in(reg) OFF9,
            tmp = out(reg) _,
            idx = out(reg) _,
            jump = out(reg) _,
            data = out(reg) _,
            prefix = out(reg) _,
            options(nostack),
        );
    }
}

/// Encode a u64 in VLQ format using x86_64 inline asm with CLZ-based length lookup.
/// NO MULTIPLICATIONS - uses precomputed tables for O(1) length determination.
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "lzcnt",
    target_feature = "bmi2",
    feature = "asm"
))]
#[inline(always)]
fn encode_vu64_impl(n: u64, out: &mut [u8; VU64_BUF_SIZE]) {
    // SAFETY: Writing to valid buffer.
    unsafe {
        core::arch::asm!(
            // Fast path for len=1 (n < 128)
            "cmp    {n:r}, 127",
            "ja     2f",
            "mov    {t:e}, {n:e}",
            "or     {t:e}, 0x80",
            "mov    byte ptr [{out:r}], {t:l}",
            "mov    qword ptr [{out:r} + 1], 0",
            "jmp    99f",

            // Main path - NO MULTIPLICATIONS
            "2:",
            "mov    {t:r}, {n:r}",
            "or     {t:r}, 1",
            "lzcnt  {t:r}, {t:r}",

            // Load base table pointer
            "lea    {base:r}, [rip + 77f]",

            // len = clz_to_len[clz]
            "movzx  {len:e}, byte ptr [{base:r} + {t:r}]",

            // Boundary check: if n >= thresholds[len], len++
            "mov    {t:r}, [{base:r} + 72 + {len:r}*8]",
            "cmp    {n:r}, {t:r}",
            "adc    {len:e}, 0",

            // Load offset and compute data = n - offset[len]
            "mov    {t:r}, [{base:r} + 152 + {len:r}*8]",
            "mov    {data:r}, {n:r}",
            "sub    {data:r}, {t:r}",

            // Load shift and compute prefix = data >> shift[len]
            "movzx  {t:e}, byte ptr [{base:r} + 232 + {len:r}]",
            "shrx   {prefix:r}, {data:r}, {t:r}",

            // Apply mask[len]
            "and    {prefix:r}, [{base:r} + 248 + {len:r}*8]",

            // Apply prefix_or[len]
            "or     {prefix:r}, [{base:r} + 328 + {len:r}*8]",

            // Write output
            "mov    byte ptr [{out:r}], {prefix:l}",
            "mov    qword ptr [{out:r} + 1], {data:r}",
            "jmp    99f",

            // Table data
            ".p2align 3",
            "77:",
            // clz_to_len[65] at offset 0 (65 bytes, padded to 72)
            ".byte 9,9,9,9,9,9,9, 8,8,8,8,8,8,8, 7,7,7,7,7,7,7, 6,6,6,6,6,6,6",
            ".byte 5,5,5,5,5,5,5, 4,4,4,4,4,4,4, 3,3,3,3,3,3,3, 2,2,2,2,2,2,2,2",
            ".byte 1,1,1,1,1,1,1,1,1",
            ".p2align 3",

            // thresholds[10] at offset 72 (80 bytes) - indexed by len (0 unused)
            ".quad 0",
            ".quad 128, 16512, 2113664, 270549120, 34630287488",
            ".quad 4432676798592, 567382630219904, 72624976668147840, 0xFFFFFFFFFFFFFFFF",

            // offsets[10] at offset 152 (80 bytes) - indexed by len (0 unused)
            ".quad 0",
            ".quad 0, 0x80, 0x4080, 0x204080, 0x10204080",
            ".quad 0x0810204080, 0x040810204080, 0x02040810204080, 0x0102040810204080",

            // shifts[10] at offset 232 (10 bytes, padded to 16)
            ".byte 0, 0, 8, 16, 24, 32, 40, 48, 56, 0",
            ".p2align 3",

            // masks[10] at offset 248 (80 bytes) - indexed by len
            ".quad 0",
            ".quad 0x7F, 0x3F, 0x1F, 0x0F, 0x07, 0x03, 0x01, 0x00, 0x00",

            // prefix_or[10] at offset 328 (80 bytes) - indexed by len
            ".quad 0",
            ".quad 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01, 0x00",

            "99:",

            n = in(reg) n,
            out = in(reg) out.as_mut_ptr(),
            t = out(reg) _,
            base = out(reg) _,
            len = out(reg) _,
            prefix = out(reg) _,
            data = out(reg) _,
            options(nostack),
        );
    }
}

/// Encode a u64 in VLQ format.
#[inline(always)]
pub fn encode_vu64(n: u64) -> Vu64 {
    #[cfg(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(
            target_arch = "x86_64",
            target_feature = "lzcnt",
            target_feature = "bmi2",
            feature = "asm"
        )
    ))]
    {
        let mut bytes = core::mem::MaybeUninit::<[u8; VU64_BUF_SIZE]>::uninit();
        // SAFETY: ASM writes 1 byte at offset 0 (prefix) and 8 bytes at offset 1 (data)
        unsafe {
            encode_vu64_impl(n, &mut *bytes.as_mut_ptr());
            Vu64(bytes.assume_init())
        }
    }

    #[cfg(not(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(
            target_arch = "x86_64",
            target_feature = "lzcnt",
            target_feature = "bmi2",
            feature = "asm"
        )
    )))]
    {
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

/// Decode a u64 from a byte slice using branchless table lookup x86_64 assembly.
/// Uses BMI2 SHLX for variable shifts. Tables are embedded inline.
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "lzcnt",
    target_feature = "bmi2",
    feature = "asm"
))]
#[inline(always)]
pub fn decode_vu64_slice(data: &[u8]) -> (u64, usize) {
    let data_len = data.len();

    if data_len == 0 {
        return (0, 0);
    }

    let value: u64;
    let len: usize;

    // SAFETY: We've verified data is not empty. Bounds checked after decode.
    unsafe {
        core::arch::asm!(
            // Load prefix byte
            "movzx  {prefix:e}, byte ptr [{ptr:r}]",

            // idx = lzcnt(prefix) - 24, len = idx + 1
            "lzcnt  {idx:e}, {prefix:e}",
            "sub    {idx:e}, 24",
            "lea    {len:e}, [{idx:e} + 1]",

            // Load 8 raw bytes from ptr+1
            "mov    {out:r}, qword ptr [{ptr:r} + 1]",

            // Table lookups via inline data
            "lea    {t:r}, [rip + 22f]",
            "and    {out:r}, [{t:r} + {idx:r}*8]",

            "lea    {t:r}, [rip + 23f]",
            "and    {prefix:r}, [{t:r} + {idx:r}*8]",

            "lea    {t:r}, [rip + 24f]",
            "mov    {t:r}, [{t:r} + {idx:r}*8]",
            "shlx   {prefix:r}, {prefix:r}, {t:r}",
            "or     {out:r}, {prefix:r}",

            "lea    {t:r}, [rip + 25f]",
            "add    {out:r}, [{t:r} + {idx:r}*8]",
            "jmp    26f",

            // Inline tables
            ".p2align 3",
            "25:",  // offsets
            ".quad 0, 0x80, 0x4080, 0x204080, 0x10204080",
            ".quad 0x0810204080, 0x040810204080, 0x02040810204080, 0x0102040810204080",

            "24:",  // shifts
            ".quad 0, 8, 16, 24, 32, 40, 48, 56, 0",

            "23:",  // pmasks
            ".quad 0x7F, 0x3F, 0x1F, 0x0F, 0x07, 0x03, 0x01, 0x00, 0x00",

            "22:",  // dmasks
            ".quad 0, 0xFF, 0xFFFF, 0xFFFFFF, 0xFFFFFFFF",
            ".quad 0xFFFFFFFFFF, 0xFFFFFFFFFFFF, 0xFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF",

            "26:",  // done

            ptr = in(reg) data.as_ptr(),
            prefix = out(reg) _,
            out = out(reg) value,
            len = out(reg) len,
            idx = out(reg) _,
            t = out(reg) _,
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
    all(
        target_arch = "x86_64",
        target_feature = "lzcnt",
        target_feature = "bmi2",
        feature = "asm"
    )
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
