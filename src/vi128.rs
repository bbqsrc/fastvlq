//! Signed 128-bit VLQ encoding (zigzag).

use core::fmt::{Debug, Display};

#[cfg(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
))]
use crate::vu128::VU128_BUF_SIZE;
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
use crate::vu128::encode_vu128;
use crate::vu128::{Vu128, decode_vu128_slice};

/// Zigzag encode a signed i128 to unsigned u128.
#[inline(always)]
pub const fn zigzag_encode_i128(n: i128) -> u128 {
    ((n << 1) ^ (n >> 127)) as u128
}

/// Zigzag decode an unsigned u128 to signed i128.
#[inline(always)]
pub const fn zigzag_decode_i128(n: u128) -> i128 {
    ((n >> 1) as i128) ^ -((n & 1) as i128)
}

/// Fused zigzag + encode for i128 using aarch64 inline asm.
/// Writes directly to output buffer.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn encode_vi128_asm(n: i128, out: &mut [u8; VU128_BUF_SIZE]) {
    let n_lo = n as u64;
    let n_hi = (n >> 64) as u64;

    // SAFETY: Writing to valid buffer.
    unsafe {
        core::arch::asm!(
            // Zigzag encode: ((n << 1) ^ (n >> 127))
            // Left shift by 1 (128-bit)
            "extr   x4, x1, x0, #63",  // x4 = new_hi = (hi << 1) | (lo >> 63)
            "lsl    x7, x0, #1",       // x7 = new_lo = lo << 1
            // Arithmetic right shift by 127 = sign bit replicated
            "asr    x2, x1, #63",      // x2 = sign (all 0s or all 1s)
            // XOR to complete zigzag
            "eor    x0, x7, x2",       // x0 = new_lo ^ sign = zigzag_lo
            "eor    x1, x4, x2",       // x1 = new_hi ^ sign = zigzag_hi

            // Now x0, x1 contain zigzag-encoded unsigned value
            // Check if high part is zero (fits in 64 bits)
            "cbnz   x1, 50f",

            // === Low 64-bit path (lengths 1-9) ===
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

            // Check for 9-byte encoding (second byte >= 0x80)
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x8102, lsl #48",
            "cmp    x0, x4",
            "b.lo   110f",

            // len=9
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "sub    x2, x0, x4",
            "mov    x5, #0x00",
            "lsr    x6, x2, #56",
            "and    x2, x2, #0x00FFFFFFFFFFFFFF",
            "mov    x3, #0",
            "b      200f",

            // len=1
            "100:",
            "orr    x5, x0, #0x80",
            "mov    x6, #0",
            "mov    x2, #0",
            "mov    x3, #0",
            "b      200f",

            // len=2
            "101:",
            "sub    x2, x0, #128",
            "lsr    x5, x2, #8",
            "orr    x5, x5, #0x40",
            "mov    x6, #0",
            "and    x2, x2, #0xFF",
            "mov    x3, #0",
            "b      200f",

            // len=3
            "102:",
            "mov    x4, #0x4080",
            "sub    x2, x0, x4",
            "lsr    x5, x2, #16",
            "orr    x5, x5, #0x20",
            "mov    x6, #0",
            "and    x2, x2, #0xFFFF",
            "mov    x3, #0",
            "b      200f",

            // len=4
            "103:",
            "mov    x4, #0x4080",
            "movk   x4, #0x20, lsl #16",
            "sub    x2, x0, x4",
            "lsr    x5, x2, #24",
            "orr    x5, x5, #0x10",
            "mov    x6, #0",
            "ubfx   x2, x2, #0, #24",
            "mov    x3, #0",
            "b      200f",

            // len=5
            "104:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "sub    x2, x0, x4",
            "lsr    x5, x2, #32",
            "orr    x5, x5, #0x08",
            "mov    x6, #0",
            "and    x2, x2, #0xFFFFFFFF",
            "mov    x3, #0",
            "b      200f",

            // len=6
            "105:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x8, lsl #32",
            "sub    x2, x0, x4",
            "lsr    x5, x2, #40",
            "orr    x5, x5, #0x04",
            "mov    x6, #0",
            "mov    x7, #0xFFFFFFFFFF",
            "and    x2, x2, x7",
            "mov    x3, #0",
            "b      200f",

            // len=7
            "106:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x408, lsl #32",
            "sub    x2, x0, x4",
            "lsr    x5, x2, #48",
            "orr    x5, x5, #0x02",
            "mov    x6, #0",
            "mov    x7, #0xFFFFFFFFFFFF",
            "and    x2, x2, x7",
            "mov    x3, #0",
            "b      200f",

            // len=8
            "107:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0002, lsl #48",
            "sub    x2, x0, x4",
            "mov    x5, #0x01",
            "mov    x6, #0",
            "mov    x7, #0xFFFFFFFFFFFFFF",
            "and    x2, x2, x7",
            "mov    x3, #0",
            "b      200f",

            // len=10 (gap between 8-byte and 9-byte)
            "110:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x5, #0",
            "mov    x6, #0x40",
            "orr    x6, x6, x2, lsr #56",
            "and    x2, x2, #0x00FFFFFFFFFFFFFF",
            "mov    x3, #0",
            "b      200f",

            // === High 64-bit path (lengths 9-18) ===
            "50:",
            "cmp    x1, #1",
            "b.hi   51f",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "cmp    x0, x4",
            "b.hs   51f",

            // n_hi=1, n_lo < offset!(9): 9-byte encoding
            "sub    x2, x0, x4",
            "mov    x5, #0x00",
            "lsr    x6, x2, #56",
            "mov    x7, #0x00FFFFFFFFFFFFFF",
            "and    x2, x2, x7",
            "mov    x3, #0",
            "b      200f",

            "51:",
            // 10+ byte encodings
            "cmp    x1, #64",
            "b.lo   150f",

            "mov    x7, #8256",
            "cmp    x1, x7",
            "b.lo   151f",

            "mov    x7, #8256",
            "movk   x7, #0x10, lsl #16",
            "cmp    x1, x7",
            "b.lo   152f",

            "mov    x7, #0x80",
            "movk   x7, #0x810, lsl #16",
            "cmp    x1, x7",
            "b.lo   153f",

            "mov    x7, #0x4000",
            "movk   x7, #0x4080, lsl #16",
            "cmp    x1, x7",
            "b.lo   154f",

            "mov    x7, #0x4000",
            "movk   x7, #0x4080, lsl #16",
            "movk   x7, #0x20, lsl #32",
            "cmp    x1, x7",
            "b.lo   155f",

            "mov    x7, #0x4000",
            "movk   x7, #0x4080, lsl #16",
            "movk   x7, #0x1020, lsl #32",
            "cmp    x1, x7",
            "b.lo   156f",

            // len=18: raw 128-bit
            "mov    x5, #0",
            "mov    x6, #0",
            "mov    x2, x0",
            "mov    x3, x1",
            "b      200f",

            // len=10
            "150:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #1",
            "sbc    x3, x1, x7",
            "mov    x5, #0",
            "lsr    x6, x3, #6",
            "orr    x6, x6, #0x40",
            "and    x3, x3, #0x3F",
            "b      200f",

            // len=11
            "151:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #65",
            "sbc    x3, x1, x7",
            "mov    x5, #0",
            "lsr    x6, x3, #13",
            "orr    x6, x6, #0x20",
            "mov    x7, #0x1FFF",
            "and    x3, x3, x7",
            "b      200f",

            // len=12
            "152:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #8256",
            "movk   x7, #0x01, lsl #16",
            "sbc    x3, x1, x7",
            "mov    x5, #0",
            "lsr    x6, x3, #20",
            "orr    x6, x6, #0x10",
            "mov    x7, #0xFFFFF",
            "and    x3, x3, x7",
            "b      200f",

            // len=13
            "153:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #0x4000",
            "movk   x7, #0x0108, lsl #16",
            "sbc    x3, x1, x7",
            "mov    x5, #0",
            "lsr    x6, x3, #27",
            "orr    x6, x6, #0x08",
            "mov    x7, #0x7FFFFFF",
            "and    x3, x3, x7",
            "b      200f",

            // len=14
            "154:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #0x4000",
            "movk   x7, #0x8408, lsl #16",
            "sbc    x3, x1, x7",
            "mov    x5, #0",
            "mov    x7, #34",
            "lsr    x6, x3, x7",
            "orr    x6, x6, #0x04",
            "mov    x7, #0x3FFFFFFFF",
            "and    x3, x3, x7",
            "b      200f",

            // len=15
            "155:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #0x4000",
            "movk   x7, #0x4208, lsl #16",
            "movk   x7, #0x04, lsl #32",
            "sbc    x3, x1, x7",
            "mov    x5, #0",
            "mov    x7, #41",
            "lsr    x6, x3, x7",
            "orr    x6, x6, #0x02",
            "mov    x7, #0x1FFFFFFFFFF",
            "and    x3, x3, x7",
            "b      200f",

            // len=16
            "156:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #0x4000",
            "movk   x7, #0x2108, lsl #16",
            "movk   x7, #0x0204, lsl #32",
            "sbc    x3, x1, x7",
            "mov    x5, #0",
            "mov    x6, #0x01",
            "b      200f",

            "200:",
            // Write to output buffer
            // p1 at offset 0
            "strb   w5, [{out}]",
            // Check if p1 == 0 (extended format where p2 is used)
            "cbnz   w5, 201f",
            // Extended format (len 9+): p2 at offset 1, data at offset 2
            "strb   w6, [{out}, #1]",
            "str    x2, [{out}, #2]",
            "str    x3, [{out}, #10]",
            "b      202f",
            "201:",
            // Standard format (len 1-8): data at offset 1
            "str    x2, [{out}, #1]",
            "202:",

            inout("x0") n_lo => _,
            inout("x1") n_hi => _,
            out("x2") _,
            out("x3") _,
            out("x4") _,
            out("x5") _,
            out("x6") _,
            out("x7") _,
            out = in(reg) out.as_mut_ptr(),
            options(nostack),
        );
    }
}

/// Encode a signed i128 using zigzag encoding to VLQ.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn encode_vi128(n: i128) -> Vi128 {
    let mut bytes = [0u8; VU128_BUF_SIZE];
    encode_vi128_asm(n, &mut bytes);
    Vi128(Vu128(bytes))
}

/// Fused zigzag + encode for i128 using x86_64 inline asm.
/// Writes directly to output buffer.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn encode_vi128_asm_x86(n: i128, out: &mut [u8; VU128_BUF_SIZE]) {
    let n_lo = n as u64;
    let n_hi = (n >> 64) as u64;

    // SAFETY: Writing to valid buffer.
    unsafe {
        core::arch::asm!(
            // Zigzag encode: ((n << 1) ^ (n >> 127))
            // Left shift by 1 (128-bit): new_lo = lo << 1, new_hi = (hi << 1) | (lo >> 63)
            "mov    {zz_lo:r}, {n_lo:r}",
            "mov    {zz_hi:r}, {n_hi:r}",
            "shld   {zz_hi:r}, {zz_lo:r}, 1",
            "shl    {zz_lo:r}, 1",
            // Arithmetic right shift by 127 = sign extension
            "mov    {sign:r}, {n_hi:r}",
            "sar    {sign:r}, 63",
            // XOR to complete zigzag
            "xor    {zz_lo:r}, {sign:r}",
            "xor    {zz_hi:r}, {sign:r}",

            // Now zz_lo, zz_hi contain zigzag-encoded unsigned value
            // Check if high part is zero (fits in 64 bits)
            "test   {zz_hi:r}, {zz_hi:r}",
            "jnz    50f",

            // === Low 64-bit path (lengths 1-9) ===
            "cmp    {zz_lo:r}, 128",
            "jb     100f",

            "cmp    {zz_lo:r}, 0x4080",
            "jb     101f",

            "cmp    {zz_lo:r}, 0x204080",
            "jb     102f",

            "mov    {tmp:r}, 0x10204080",
            "cmp    {zz_lo:r}, {tmp:r}",
            "jb     103f",

            "mov    {tmp:r}, 0x810204080",
            "cmp    {zz_lo:r}, {tmp:r}",
            "jb     104f",

            "mov    {tmp:r}, 0x40810204080",
            "cmp    {zz_lo:r}, {tmp:r}",
            "jb     105f",

            "mov    {tmp:r}, 0x2040810204080",
            "cmp    {zz_lo:r}, {tmp:r}",
            "jb     106f",

            "mov    {tmp:r}, 0x102040810204080",
            "cmp    {zz_lo:r}, {tmp:r}",
            "jb     107f",

            // Check for 9-byte encoding (second byte >= 0x80)
            "mov    {tmp:r}, 0x8102040810204080",
            "cmp    {zz_lo:r}, {tmp:r}",
            "jb     110f",

            // len=9
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {prefix1:r}, 0x00",
            "mov    {prefix2:r}, {data_lo:r}",
            "shr    {prefix2:r}, 56",
            "mov    {tmp:r}, 0x00FFFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=1
            "100:",
            "mov    {prefix1:r}, {zz_lo:r}",
            "or     {prefix1:r}, 0x80",
            "xor    {prefix2:r}, {prefix2:r}",
            "xor    {data_lo:r}, {data_lo:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=2
            "101:",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, 128",
            "mov    {prefix1:r}, {data_lo:r}",
            "shr    {prefix1:r}, 8",
            "or     {prefix1:r}, 0x40",
            "xor    {prefix2:r}, {prefix2:r}",
            "and    {data_lo:r}, 0xFF",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=3
            "102:",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, 0x4080",
            "mov    {prefix1:r}, {data_lo:r}",
            "shr    {prefix1:r}, 16",
            "or     {prefix1:r}, 0x20",
            "xor    {prefix2:r}, {prefix2:r}",
            "and    {data_lo:r}, 0xFFFF",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=4
            "103:",
            "mov    {tmp:r}, 0x204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {prefix1:r}, {data_lo:r}",
            "shr    {prefix1:r}, 24",
            "or     {prefix1:r}, 0x10",
            "xor    {prefix2:r}, {prefix2:r}",
            "and    {data_lo:r}, 0xFFFFFF",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=5
            "104:",
            "mov    {tmp:r}, 0x10204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {prefix1:r}, {data_lo:r}",
            "shr    {prefix1:r}, 32",
            "or     {prefix1:r}, 0x08",
            "xor    {prefix2:r}, {prefix2:r}",
            "mov    {tmp:r}, 0xFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=6
            "105:",
            "mov    {tmp:r}, 0x810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {prefix1:r}, {data_lo:r}",
            "shr    {prefix1:r}, 40",
            "or     {prefix1:r}, 0x04",
            "xor    {prefix2:r}, {prefix2:r}",
            "mov    {tmp:r}, 0xFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=7
            "106:",
            "mov    {tmp:r}, 0x40810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {prefix1:r}, {data_lo:r}",
            "shr    {prefix1:r}, 48",
            "or     {prefix1:r}, 0x02",
            "xor    {prefix2:r}, {prefix2:r}",
            "mov    {tmp:r}, 0xFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=8
            "107:",
            "mov    {tmp:r}, 0x2040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {prefix1:r}, 0x01",
            "xor    {prefix2:r}, {prefix2:r}",
            "mov    {tmp:r}, 0xFFFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=10 (gap between 8-byte and 9-byte)
            "110:",
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, 0x40",
            "mov    {tmp:r}, {data_lo:r}",
            "shr    {tmp:r}, 56",
            "or     {prefix2:r}, {tmp:r}",
            "mov    {tmp:r}, 0x00FFFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // === High 64-bit path (lengths 9-18) ===
            "50:",
            "cmp    {zz_hi:r}, 1",
            "ja     51f",
            "mov    {tmp:r}, 0x102040810204080",
            "cmp    {zz_lo:r}, {tmp:r}",
            "jae    51f",

            // n_hi=1, n_lo < offset!(9): 9-byte encoding
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {prefix1:r}, 0x00",
            "mov    {prefix2:r}, {data_lo:r}",
            "shr    {prefix2:r}, 56",
            "mov    {tmp:r}, 0x00FFFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            "51:",
            // 10+ byte encodings
            "cmp    {zz_hi:r}, 64",
            "jb     150f",

            "mov    {tmp:r}, 8256",
            "cmp    {zz_hi:r}, {tmp:r}",
            "jb     151f",

            "mov    {tmp:r}, 0x100000 + 8256",
            "cmp    {zz_hi:r}, {tmp:r}",
            "jb     152f",

            "mov    {tmp:r}, 0x8100080",
            "cmp    {zz_hi:r}, {tmp:r}",
            "jb     153f",

            "mov    {tmp:r}, 0x408040004000",
            "cmp    {zz_hi:r}, {tmp:r}",
            "jb     154f",

            "mov    {tmp:r}, 0x20408040004000",
            "cmp    {zz_hi:r}, {tmp:r}",
            "jb     155f",

            "mov    {tmp:r}, 0x1020408040004000",
            "cmp    {zz_hi:r}, {tmp:r}",
            "jb     156f",

            // len=18: raw 128-bit
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, 0",
            "mov    {data_lo:r}, {zz_lo:r}",
            "mov    {data_hi:r}, {zz_hi:r}",
            "jmp    200f",

            // len=10
            "150:",
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {zz_hi:r}",
            "sbb    {data_hi:r}, 1",
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, {data_hi:r}",
            "shr    {prefix2:r}, 6",
            "or     {prefix2:r}, 0x40",
            "and    {data_hi:r}, 0x3F",
            "jmp    200f",

            // len=11
            "151:",
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {zz_hi:r}",
            "sbb    {data_hi:r}, 65",
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, {data_hi:r}",
            "shr    {prefix2:r}, 13",
            "or     {prefix2:r}, 0x20",
            "and    {data_hi:r}, 0x1FFF",
            "jmp    200f",

            // len=12
            "152:",
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {tmp:r}, 0x108240",
            "mov    {data_hi:r}, {zz_hi:r}",
            "sbb    {data_hi:r}, {tmp:r}",
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, {data_hi:r}",
            "shr    {prefix2:r}, 20",
            "or     {prefix2:r}, 0x10",
            "mov    {tmp:r}, 0xFFFFF",
            "and    {data_hi:r}, {tmp:r}",
            "jmp    200f",

            // len=13
            "153:",
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {tmp:r}, 0x10840004000",
            "mov    {data_hi:r}, {zz_hi:r}",
            "sbb    {data_hi:r}, {tmp:r}",
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, {data_hi:r}",
            "shr    {prefix2:r}, 27",
            "or     {prefix2:r}, 0x08",
            "mov    {tmp:r}, 0x7FFFFFF",
            "and    {data_hi:r}, {tmp:r}",
            "jmp    200f",

            // len=14
            "154:",
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {tmp:r}, 0x842040804004000",
            "mov    {data_hi:r}, {zz_hi:r}",
            "sbb    {data_hi:r}, {tmp:r}",
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, {data_hi:r}",
            "mov    {tmp:e}, 34",
            "shr    {prefix2:r}, cl",
            "or     {prefix2:r}, 0x04",
            "mov    {tmp:r}, 0x3FFFFFFFF",
            "and    {data_hi:r}, {tmp:r}",
            "jmp    200f",

            // len=15
            "155:",
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {tmp:r}, 0x42104284004000",
            "mov    {data_hi:r}, {zz_hi:r}",
            "sbb    {data_hi:r}, {tmp:r}",
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, {data_hi:r}",
            "mov    {tmp:e}, 41",
            "shr    {prefix2:r}, cl",
            "or     {prefix2:r}, 0x02",
            "mov    {tmp:r}, 0x1FFFFFFFFFF",
            "and    {data_hi:r}, {tmp:r}",
            "jmp    200f",

            // len=16
            "156:",
            "mov    {tmp:r}, 0x102040810204080",
            "mov    {data_lo:r}, {zz_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {tmp:r}, 0x2108214284004000",
            "mov    {data_hi:r}, {zz_hi:r}",
            "sbb    {data_hi:r}, {tmp:r}",
            "mov    {prefix1:r}, 0",
            "mov    {prefix2:r}, 0x01",
            "jmp    200f",

            "200:",
            // Write to output buffer
            // p1 at offset 0
            "mov    byte ptr [{out}], {prefix1:l}",
            // Check if p1 == 0 (extended format where p2 is used)
            "test   {prefix1:l}, {prefix1:l}",
            "jnz    201f",
            // Extended format (len 9+): p2 at offset 1, data at offset 2
            "mov    byte ptr [{out} + 1], {prefix2:l}",
            "mov    qword ptr [{out} + 2], {data_lo:r}",
            "mov    qword ptr [{out} + 10], {data_hi:r}",
            "jmp    202f",
            "201:",
            // Standard format (len 1-8): data at offset 1
            "mov    qword ptr [{out} + 1], {data_lo:r}",
            "202:",

            n_lo = in(reg) n_lo,
            n_hi = in(reg) n_hi,
            zz_lo = out(reg) _,
            zz_hi = out(reg) _,
            sign = out(reg) _,
            tmp = out(reg) _,
            prefix1 = out(reg) _,
            prefix2 = out(reg) _,
            data_lo = out(reg) _,
            data_hi = out(reg) _,
            out = in(reg) out.as_mut_ptr(),
            options(nostack),
        );
    }
}

/// Encode a signed i128 using zigzag encoding to VLQ.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn encode_vi128(n: i128) -> Vi128 {
    let mut bytes = [0u8; VU128_BUF_SIZE];
    encode_vi128_asm_x86(n, &mut bytes);
    Vi128(Vu128(bytes))
}

/// Encode a signed i128 using zigzag encoding to VLQ.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
#[inline(always)]
pub fn encode_vi128(n: i128) -> Vi128 {
    Vi128(encode_vu128(zigzag_encode_i128(n)))
}

/// Decode a Vi128 back to a native i128.
#[inline(always)]
pub fn decode_vi128(n: Vi128) -> i128 {
    n.get()
}

/// Decode a Vi128 from a byte slice.
///
/// Returns (value, bytes_consumed) on success, or (0, 0) if the slice is empty/invalid.
#[inline(always)]
pub fn decode_vi128_slice(data: &[u8]) -> (i128, usize) {
    let (unsigned, len) = decode_vu128_slice(data);
    if len == 0 {
        return (0, 0);
    }
    (zigzag_decode_i128(unsigned), len)
}

/// A signed 128-bit integer in value-length quantity encoding using zigzag.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Vi128(Vu128);

#[allow(clippy::len_without_is_empty)]
impl Vi128 {
    /// Construct a new VLQ instance from the given `i128`.
    #[inline(always)]
    pub fn new(value: i128) -> Vi128 {
        encode_vi128(value)
    }

    /// Retrieve the stored number as `i128`.
    #[inline(always)]
    pub fn get(&self) -> i128 {
        zigzag_decode_i128(self.0.get())
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

impl From<i128> for Vi128 {
    fn from(n: i128) -> Self {
        encode_vi128(n)
    }
}

impl From<Vi128> for i128 {
    fn from(n: Vi128) -> Self {
        decode_vi128(n)
    }
}

impl Display for Vi128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&i128::from(*self), f)
    }
}

impl Debug for Vi128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.0.bytes();
        write!(f, "Vi128(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
