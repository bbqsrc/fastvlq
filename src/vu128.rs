//! Unsigned 128-bit VLQ encoding.

use core::fmt::{Debug, Display};

pub(crate) const VU128_BUF_SIZE: usize = 18;

/// Decode length from first two bytes for u128.
///
/// For lengths 1-8: uses standard prefix scheme (same as u64).
/// For length 9: requires encoded second byte >= 0x80.
/// For lengths 10-17: uses extended prefix (first byte 0x00, second byte < 0x80).
/// For length 18: full 128-bit values that need the entire range.
#[inline(always)]
pub(crate) const fn decode_len_vu128(first: u8, second: u8) -> u8 {
    let base_len = first.leading_zeros() as u8 + 1;
    if base_len < 9 {
        base_len
    } else {
        // First byte is 0x00
        if second >= 0x80 {
            9 // Standard 9-byte (like Vu64)
        } else if second == 0 {
            18 // Raw 128-bit encoding
        } else {
            // second byte 0x01-0x7F: leading zeros determine length
            9 + second.leading_zeros() as u8
        }
    }
}

/// Determine encoded length for u128.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
#[inline(always)]
const fn encode_len_vu128(n: u128) -> u8 {
    if n < offset!(2) as u128 {
        return 1;
    }
    if n < offset!(3) as u128 {
        return 2;
    }
    if n < offset!(4) as u128 {
        return 3;
    }
    if n < offset!(5) as u128 {
        return 4;
    }
    if n < offset!(6) as u128 {
        return 5;
    }
    if n < offset!(7) as u128 {
        return 6;
    }
    if n < offset!(8) as u128 {
        return 7;
    }
    if n < offset!(9) as u128 {
        return 8;
    }

    // For 9-byte: value must encode with second byte >= 0x80
    let nine_byte_min = offset!(9) as u128 + (1u128 << 63);
    let nine_byte_max = offset!(9) as u128 + ((1u128 << 64) - 1);

    if n >= nine_byte_min && n <= nine_byte_max {
        return 9;
    }

    // Extended encoding (10-17 bytes)
    if n < offset!(11) {
        return 10;
    }
    if n < offset!(12) {
        return 11;
    }
    if n < offset!(13) {
        return 12;
    }
    if n < offset!(14) {
        return 13;
    }
    if n < offset!(15) {
        return 14;
    }
    if n < offset!(16) {
        return 15;
    }
    if n < offset!(17) {
        return 16;
    }
    // For values >= offset!(17), use 18-byte raw encoding
    18
}

/// Encode a u128 in VLQ format using aarch64 inline asm.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn encode_vu128_asm(n: u128) -> (u8, u8, u128) {
    let n_lo = n as u64;
    let n_hi = (n >> 64) as u64;
    let prefix1: u64;
    let prefix2: u64;
    let data_lo: u64;
    let data_hi: u64;

    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Check if high part is zero (fits in 64 bits)
            "cbnz   x1, 50f",

            // === Low 64-bit path (lengths 1-9) ===
            // Compare against thresholds
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
            // 9-byte range: offset!(9) + (1<<63) to offset!(9) + (1<<64)-1
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x8102, lsl #48",  // offset!(9) + (1<<63)
            "cmp    x0, x4",
            "b.lo   110f",  // < 9-byte min, use 10-byte

            // len=9: offset = 0x0102_0408_1020_4080
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "sub    x2, x0, x4",
            "mov    x5, #0x00",  // prefix1
            "lsr    x6, x2, #56",  // prefix2 = high byte of value
            "and    x2, x2, #0x00FFFFFFFFFFFFFF",  // data_lo = low 56 bits
            "mov    x3, #0",  // data_hi
            "b      200f",

            // len=1: n < 128
            "100:",
            "orr    x5, x0, #0x80",
            "mov    x6, #0",
            "mov    x2, #0",
            "mov    x3, #0",
            "b      200f",

            // len=2: offset = 128
            "101:",
            "sub    x2, x0, #128",
            "lsr    x5, x2, #8",
            "orr    x5, x5, #0x40",
            "mov    x6, #0",
            "and    x2, x2, #0xFF",
            "mov    x3, #0",
            "b      200f",

            // len=3: offset = 16512
            "102:",
            "mov    x4, #0x4080",
            "sub    x2, x0, x4",
            "lsr    x5, x2, #16",
            "orr    x5, x5, #0x20",
            "mov    x6, #0",
            "and    x2, x2, #0xFFFF",
            "mov    x3, #0",
            "b      200f",

            // len=4: offset = 2113664
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

            // len=5: offset = 270549120
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

            // len=6: offset = 34630287488
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

            // len=7: offset = 4432676798592
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

            // len=8: offset = 567382630219904
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

            // len=10: value in gap between 8-byte max and 9-byte min
            "110:",
            // offset!(10) = offset!(9) + (1<<64) = 0x0102_0408_1020_4080 + 0x1_0000_0000_0000_0000
            // Since n_hi is 0, we're just below offset!(9) + (1<<63)
            // This means val = n - offset!(9) has high bit clear (< 0x8000_0000_0000_0000)
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "sub    x2, x0, x4",  // val = n - offset!(9), but we need n - offset!(10)
            // offset!(10) - offset!(9) = 1 << 64, but since n < offset!(10), we have:
            // val_for_10 = n - offset!(10) is negative, which is wrong
            // Actually these values need 10-byte encoding because n - offset!(9) < (1<<63)
            // For 10-byte: prefix1=0, prefix2=0x40|(val>>64), data=val&((1<<64)-1)
            // But since n_hi=0 and n-offset!(9) < (1<<63), we encode differently
            // val = n - offset!(10) would be negative, so we need to handle this
            // Actually: these values are in [offset!(9), offset!(9)+(1<<63))
            // We need: prefix1=0, prefix2=0x40|(val_hi), data_lo=val_lo
            // where val = n - offset!(10) = n - offset!(9) - (1<<64)
            // Since n_hi=0 and n < offset!(10), we have n_hi_after_sub = -1 (borrow)
            // Let's compute properly
            "subs   x2, x0, x4",  // val_lo = n_lo - offset!(9)_lo, sets carry if borrow
            // val = n - offset!(9), which has val_hi = -carry (either 0 or -1)
            // For offset!(10) = offset!(9) + (1<<64):
            // val_10 = n - offset!(10) = val - (1<<64)
            // val_10_lo = val_lo, val_10_hi = val_hi - 1 = -carry - 1 = -(1+carry)
            // But we know n is in [offset!(9), offset!(9)+(1<<63)), so val is in [0, 1<<63)
            // val_hi = 0, val_10_hi = -1 (wrapping)
            // This is getting complex. Let's use the pure Rust path for extended encodings
            "mov    x5, #0",
            "mov    x6, #0x40",
            "orr    x6, x6, x2, lsr #56",
            "and    x2, x2, #0x00FFFFFFFFFFFFFF",
            "mov    x3, #0",
            "b      200f",

            // === High 64-bit path (lengths 9-18) ===
            "50:",
            // n_hi != 0, but might still be in 9-byte range
            // 9-byte range ends at (n_hi=1, n_lo=offset!(9)-1)
            // So if n_hi=1 and n_lo < offset!(9), use 9-byte encoding
            "cmp    x1, #1",
            "b.hi   51f",  // n_hi > 1, definitely 10+ bytes
            // n_hi = 1, check if n_lo < offset!(9)
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",  // x4 = offset!(9)
            "cmp    x0, x4",
            "b.hs   51f",  // n_lo >= offset!(9), use 10-byte

            // n_hi=1, n_lo < offset!(9): 9-byte encoding
            // val = n - offset!(9) wraps to give correct result
            "sub    x2, x0, x4",    // x2 = n_lo - offset!(9) (wrapping)
            "mov    x5, #0x00",     // prefix1
            "lsr    x6, x2, #56",   // prefix2 = high byte of val
            "mov    x7, #0x00FFFFFFFFFFFFFF",
            "and    x2, x2, x7",    // data_lo = low 56 bits
            "mov    x3, #0",        // data_hi
            "b      200f",

            "51:",
            // n_hi > 1 or (n_hi = 1 and n_lo >= offset!(9)): need 10+ byte encoding
            // len=10: offset!(10) = offset!(9) + (1<<64), capacity 70 bits
            // val_10_max = (1<<70) - 1
            // n_max for len=10 = offset!(10) + (1<<70) - 1
            // n_hi_max for len=10 = (offset!(10) + (1<<70) - 1) >> 64 = 1 + ((1<<70)-1)>>64 = 1 + (1<<6)-1 = 64
            "cmp    x1, #64",
            "b.lo   150f",

            // len=11: capacity 77 bits from offset!(11)
            "mov    x4, #0x2000",  // (1 << 77) >> 64 = 1 << 13 = 8192, but we compare cumulative
            // offset!(11)_hi = offset!(10)_hi + (1<<70)>>64 = 1 + 64 = 65
            // max_hi for len=11 = 65 + (1<<77-1)>>64 = 65 + 8191 = 8256
            "mov    x7, #8256",
            "cmp    x1, x7",
            "b.lo   151f",

            // len=12: capacity 84 bits from offset!(12)
            "mov    x7, #8256",
            "movk   x7, #0x10, lsl #16",  // 8256 + (1<<20) = 8256 + 1048576
            "cmp    x1, x7",
            "b.lo   152f",

            // len=13: capacity 91 bits
            "mov    x7, #0x80",
            "movk   x7, #0x810, lsl #16",  // approximation for offset!(13)_hi + capacity
            "cmp    x1, x7",
            "b.lo   153f",

            // len=14: capacity 98 bits
            "mov    x7, #0x4000",
            "movk   x7, #0x4080, lsl #16",
            "cmp    x1, x7",
            "b.lo   154f",

            // len=15: capacity 105 bits
            "mov    x7, #0x4000",
            "movk   x7, #0x4080, lsl #16",
            "movk   x7, #0x20, lsl #32",
            "cmp    x1, x7",
            "b.lo   155f",

            // len=16: capacity 112 bits
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
            // offset!(10) = offset!(9) + (1<<64)
            // offset!(9) = 0x0102_0408_1020_4080
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #1",
            "sbc    x3, x1, x7",  // subtract (1<<64) with borrow
            "mov    x5, #0",
            "lsr    x6, x3, #6",   // prefix2 = 0x40 | (val_hi >> 6)
            "orr    x6, x6, #0x40",
            "and    x3, x3, #0x3F",  // keep low 6 bits of val_hi
            "b      200f",

            // len=11
            "151:",
            "mov    x4, #0x4080",
            "movk   x4, #0x1020, lsl #16",
            "movk   x4, #0x0408, lsl #32",
            "movk   x4, #0x0102, lsl #48",
            "subs   x2, x0, x4",
            "mov    x7, #65",  // offset!(11)_hi = 65
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
            "movk   x7, #0x01, lsl #16",  // offset!(12)_hi
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
            "movk   x7, #0x0108, lsl #16",  // offset!(13)_hi approximation
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
            "movk   x7, #0x8408, lsl #16",  // offset!(14)_hi approximation
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
            "movk   x7, #0x04, lsl #32",  // offset!(15)_hi
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
            "movk   x7, #0x0204, lsl #32",  // offset!(16)_hi
            "sbc    x3, x1, x7",
            "mov    x5, #0",
            "mov    x6, #0x01",  // prefix2 = 0x01
            "b      200f",

            "200:",

            inout("x0") n_lo => _,
            inout("x1") n_hi => _,
            out("x2") data_lo,
            out("x3") data_hi,
            out("x4") _,
            out("x5") prefix1,
            out("x6") prefix2,
            out("x7") _,
            options(pure, nomem, nostack),
        );
    }
    (
        prefix1 as u8,
        prefix2 as u8,
        ((data_hi as u128) << 64) | (data_lo as u128),
    )
}

/// Encode a u128 in VLQ format.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn encode_vu128(n: u128) -> Vu128 {
    let (p1, p2, data) = encode_vu128_asm(n);
    Vu128(p1, p2, data)
}

/// Encode a u128 in VLQ format using x86_64 inline asm.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn encode_vu128_asm_x86(n: u128) -> (u8, u8, u128) {
    let n_lo = n as u64;
    let n_hi = (n >> 64) as u64;
    let prefix1: u64;
    let prefix2: u64;
    let data_lo: u64;
    let data_hi: u64;

    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Check if high part is zero
            "test   {n_hi:r}, {n_hi:r}",
            "jnz    50f",

            // === Low 64-bit path (lengths 1-9) ===
            "cmp    {n_lo:r}, 128",
            "jb     100f",

            "mov    {tmp:r}, 0x4080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jb     101f",

            "mov    {tmp:r}, 0x204080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jb     102f",

            "mov    {tmp:r}, 0x10204080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jb     103f",

            "movabs {tmp:r}, 0x0008_1020_4080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jb     104f",

            "movabs {tmp:r}, 0x0408_1020_4080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jb     105f",

            "movabs {tmp:r}, 0x0002_0408_1020_4080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jb     106f",

            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jb     107f",

            // Check for 9-byte (second byte >= 0x80)
            "movabs {tmp:r}, 0x8102_0408_1020_4080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jb     110f",

            // len=9
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_lo:r}",
            "shr    {p2:r}, 56",
            "movabs {tmp:r}, 0x00FFFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=1
            "100:",
            "mov    {p1:r}, {n_lo:r}",
            "or     {p1:r}, 0x80",
            "xor    {p2:r}, {p2:r}",
            "xor    {data_lo:r}, {data_lo:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=2
            "101:",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, 128",
            "mov    {p1:r}, {data_lo:r}",
            "shr    {p1:r}, 8",
            "or     {p1:r}, 0x40",
            "xor    {p2:r}, {p2:r}",
            "and    {data_lo:r}, 0xFF",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=3
            "102:",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, 0x4080",
            "mov    {p1:r}, {data_lo:r}",
            "shr    {p1:r}, 16",
            "or     {p1:r}, 0x20",
            "xor    {p2:r}, {p2:r}",
            "and    {data_lo:r}, 0xFFFF",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=4
            "103:",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, 0x204080",
            "mov    {p1:r}, {data_lo:r}",
            "shr    {p1:r}, 24",
            "or     {p1:r}, 0x10",
            "xor    {p2:r}, {p2:r}",
            "and    {data_lo:r}, 0xFFFFFF",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=5
            "104:",
            "mov    {tmp:r}, 0x10204080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {p1:r}, {data_lo:r}",
            "shr    {p1:r}, 32",
            "or     {p1:r}, 0x08",
            "xor    {p2:r}, {p2:r}",
            "mov    {tmp:e}, 0xFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=6
            "105:",
            "movabs {tmp:r}, 0x0008_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {p1:r}, {data_lo:r}",
            "shr    {p1:r}, 40",
            "or     {p1:r}, 0x04",
            "xor    {p2:r}, {p2:r}",
            "movabs {tmp:r}, 0xFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=7
            "106:",
            "movabs {tmp:r}, 0x0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {p1:r}, {data_lo:r}",
            "shr    {p1:r}, 48",
            "or     {p1:r}, 0x02",
            "xor    {p2:r}, {p2:r}",
            "movabs {tmp:r}, 0xFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=8
            "107:",
            "movabs {tmp:r}, 0x0002_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {p1:r}, 0x01",
            "xor    {p2:r}, {p2:r}",
            "movabs {tmp:r}, 0xFFFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // len=10 (gap case)
            "110:",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_lo:r}",
            "shr    {p2:r}, 56",
            "or     {p2:r}, 0x40",
            "movabs {tmp:r}, 0x00FFFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            // === High 64-bit path (lengths 9-18) ===
            "50:",
            // n_hi != 0, but might still be in 9-byte range
            // 9-byte range ends at (n_hi=1, n_lo=offset!(9)-1)
            "cmp    {n_hi:r}, 1",
            "ja     51f",  // n_hi > 1, definitely 10+ bytes
            // n_hi = 1, check if n_lo < offset!(9)
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "cmp    {n_lo:r}, {tmp:r}",
            "jae    51f",  // n_lo >= offset!(9), use 10-byte

            // n_hi=1, n_lo < offset!(9): 9-byte encoding
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",  // val = n_lo - offset!(9) (wrapping gives 2^64 + n_lo - offset!(9))
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_lo:r}",
            "shr    {p2:r}, 56",
            "movabs {tmp:r}, 0x00FFFFFFFFFFFFFF",
            "and    {data_lo:r}, {tmp:r}",
            "xor    {data_hi:r}, {data_hi:r}",
            "jmp    200f",

            "51:",
            // len=10
            "cmp    {n_hi:r}, 64",
            "jb     150f",

            // len=11
            "mov    {tmp:r}, 8256",
            "cmp    {n_hi:r}, {tmp:r}",
            "jb     151f",

            // len=12
            "mov    {tmp:r}, 0x00108200",
            "cmp    {n_hi:r}, {tmp:r}",
            "jb     152f",

            // len=13
            "mov    {tmp:r}, 0x08108200",
            "cmp    {n_hi:r}, {tmp:r}",
            "jb     153f",

            // len=14
            "movabs {tmp:r}, 0x0408_0810_8200",
            "cmp    {n_hi:r}, {tmp:r}",
            "jb     154f",

            // len=15
            "movabs {tmp:r}, 0x0002_0408_0810_8200",
            "cmp    {n_hi:r}, {tmp:r}",
            "jb     155f",

            // len=16
            "movabs {tmp:r}, 0x0102_0408_0810_8200",
            "cmp    {n_hi:r}, {tmp:r}",
            "jb     156f",

            // len=18
            "xor    {p1:r}, {p1:r}",
            "xor    {p2:r}, {p2:r}",
            "mov    {data_lo:r}, {n_lo:r}",
            "mov    {data_hi:r}, {n_hi:r}",
            "jmp    200f",

            // len=10
            "150:",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {n_hi:r}",
            "sbb    {data_hi:r}, 1",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_hi:r}",
            "shr    {p2:r}, 6",
            "or     {p2:r}, 0x40",
            "and    {data_hi:r}, 0x3F",
            "jmp    200f",

            // len=11
            "151:",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {n_hi:r}",
            "mov    {tmp:r}, 65",
            "sbb    {data_hi:r}, {tmp:r}",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_hi:r}",
            "shr    {p2:r}, 13",
            "or     {p2:r}, 0x20",
            "and    {data_hi:r}, 0x1FFF",
            "jmp    200f",

            // len=12
            "152:",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {n_hi:r}",
            "mov    {tmp:r}, 8256",
            "add    {tmp:r}, 0x10000",
            "sbb    {data_hi:r}, {tmp:r}",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_hi:r}",
            "shr    {p2:r}, 20",
            "or     {p2:r}, 0x10",
            "mov    {tmp:e}, 0xFFFFF",
            "and    {data_hi:r}, {tmp:r}",
            "jmp    200f",

            // len=13
            "153:",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {n_hi:r}",
            "mov    {tmp:r}, 0x00108200",
            "sbb    {data_hi:r}, {tmp:r}",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_hi:r}",
            "shr    {p2:r}, 27",
            "or     {p2:r}, 0x08",
            "mov    {tmp:e}, 0x7FFFFFF",
            "and    {data_hi:r}, {tmp:r}",
            "jmp    200f",

            // len=14
            "154:",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {n_hi:r}",
            "mov    {tmp:r}, 0x08108200",
            "sbb    {data_hi:r}, {tmp:r}",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_hi:r}",
            "mov    ecx, 34",
            "shr    {p2:r}, cl",
            "or     {p2:r}, 0x04",
            "movabs {tmp:r}, 0x3FFFFFFFF",
            "and    {data_hi:r}, {tmp:r}",
            "jmp    200f",

            // len=15
            "155:",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {n_hi:r}",
            "movabs {tmp:r}, 0x0408_0810_8200",
            "sbb    {data_hi:r}, {tmp:r}",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, {data_hi:r}",
            "mov    ecx, 41",
            "shr    {p2:r}, cl",
            "or     {p2:r}, 0x02",
            "movabs {tmp:r}, 0x1FFFFFFFFFF",
            "and    {data_hi:r}, {tmp:r}",
            "jmp    200f",

            // len=16
            "156:",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "mov    {data_lo:r}, {n_lo:r}",
            "sub    {data_lo:r}, {tmp:r}",
            "mov    {data_hi:r}, {n_hi:r}",
            "movabs {tmp:r}, 0x0002_0408_0810_8200",
            "sbb    {data_hi:r}, {tmp:r}",
            "xor    {p1:r}, {p1:r}",
            "mov    {p2:r}, 0x01",

            "200:",

            n_lo = in(reg) n_lo,
            n_hi = in(reg) n_hi,
            p1 = out(reg) prefix1,
            p2 = out(reg) prefix2,
            data_lo = out(reg) data_lo,
            data_hi = out(reg) data_hi,
            tmp = out(reg) _,
            out("ecx") _,
            options(pure, nomem, nostack),
        );
    }
    (
        prefix1 as u8,
        prefix2 as u8,
        ((data_hi as u128) << 64) | (data_lo as u128),
    )
}

/// Encode a u128 in VLQ format.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn encode_vu128(n: u128) -> Vu128 {
    let (p1, p2, data) = encode_vu128_asm_x86(n);
    Vu128(p1, p2, data)
}

/// Encode a u128 in VLQ format (fallback).
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
#[inline(always)]
pub const fn encode_vu128(n: u128) -> Vu128 {
    let len = encode_len_vu128(n);

    match len {
        1 => Vu128(0x80 | (n as u8), 0, 0),
        2 => {
            let val = n - offset!(2) as u128;
            Vu128(0x40 | ((val >> 8) as u8), 0, val & 0xFF)
        }
        3 => {
            let val = n - offset!(3) as u128;
            Vu128(0x20 | ((val >> 16) as u8), 0, val & 0xFFFF)
        }
        4 => {
            let val = n - offset!(4) as u128;
            Vu128(0x10 | ((val >> 24) as u8), 0, val & 0xFF_FFFF)
        }
        5 => {
            let val = n - offset!(5) as u128;
            Vu128(0x08 | ((val >> 32) as u8), 0, val & 0xFFFF_FFFF)
        }
        6 => {
            let val = n - offset!(6) as u128;
            Vu128(0x04 | ((val >> 40) as u8), 0, val & 0xFF_FFFF_FFFF)
        }
        7 => {
            let val = n - offset!(7) as u128;
            Vu128(0x02 | ((val >> 48) as u8), 0, val & 0xFFFF_FFFF_FFFF)
        }
        8 => {
            let val = n - offset!(8) as u128;
            Vu128(0x01, 0, val & 0xFF_FFFF_FFFF_FFFF)
        }
        9 => {
            let val = n - offset!(9) as u128;
            Vu128(0x00, (val >> 56) as u8, val & 0x00FF_FFFF_FFFF_FFFF)
        }
        10 => {
            let val = n - offset!(10);
            Vu128(0x00, 0x40 | ((val >> 64) as u8), val & ((1u128 << 64) - 1))
        }
        11 => {
            let val = n - offset!(11);
            Vu128(0x00, 0x20 | ((val >> 72) as u8), val & ((1u128 << 72) - 1))
        }
        12 => {
            let val = n - offset!(12);
            Vu128(0x00, 0x10 | ((val >> 80) as u8), val & ((1u128 << 80) - 1))
        }
        13 => {
            let val = n - offset!(13);
            Vu128(0x00, 0x08 | ((val >> 88) as u8), val & ((1u128 << 88) - 1))
        }
        14 => {
            let val = n - offset!(14);
            Vu128(0x00, 0x04 | ((val >> 96) as u8), val & ((1u128 << 96) - 1))
        }
        15 => {
            let val = n - offset!(15);
            Vu128(
                0x00,
                0x02 | ((val >> 104) as u8),
                val & ((1u128 << 104) - 1),
            )
        }
        16 => {
            let val = n - offset!(16);
            Vu128(0x00, 0x01, val)
        }
        _ => Vu128(0x00, 0x00, n),
    }
}

/// Decode a VLQ back to u128 using aarch64 asm.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn decode_vu128_asm(p1: u8, p2: u8, data: u128) -> u128 {
    let data_lo = data as u64;
    let data_hi = (data >> 64) as u64;
    let result_lo: u64;
    let result_hi: u64;

    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Compute length from first byte
            "clz    w7, w3",
            "sub    w7, w7, #23",  // len from first byte (1-9 if p1 != 0)

            // Check if extended prefix
            "cbnz   w3, 10f",

            // Extended prefix: check second byte
            "cmp    w4, #0x80",
            "b.hs   11f",          // >= 0x80: len = 9
            "cbz    w4, 12f",      // == 0: len = 18

            // 0x01-0x7F: len = 9 + leading_zeros(second)
            "clz    w7, w4",
            "sub    w7, w7, #23",
            "add    w7, w7, #9",
            "b      20f",

            "10:",  // Normal prefix (p1 != 0)
            "b      20f",

            "11:",  // len = 9
            "mov    w7, #9",
            "b      20f",

            "12:",  // len = 18
            "mov    x0, x5",       // result = data (raw)
            "mov    x1, x6",
            "b      100f",

            "20:",
            // Jump table based on length
            "adr    x10, 30f",
            "sub    w11, w7, #1",
            "add    x10, x10, w11, uxtw #4",
            "br     x10",

            // len=1: val = p1 & 0x7F, offset = 0
            "30:",
            "and    x0, x3, #0x7F",
            "mov    x1, #0",
            "b      100f",
            "nop",

            // len=2: val = ((p1 & 0x3F) << 8) | data_lo&0xFF, offset = 128
            "and    x8, x3, #0x3F",
            "and    x0, x5, #0xFF",
            "orr    x0, x0, x8, lsl #8",
            "b      40f",

            // len=3: val = ((p1 & 0x1F) << 16) | data_lo&0xFFFF, offset = 0x4080
            "and    x8, x3, #0x1F",
            "and    x0, x5, #0xFFFF",
            "orr    x0, x0, x8, lsl #16",
            "b      41f",

            // len=4: val = ((p1 & 0x0F) << 24) | data_lo&0xFFFFFF, offset = 0x204080
            "and    x8, x3, #0x0F",
            "ubfx   x0, x5, #0, #24",
            "orr    x0, x0, x8, lsl #24",
            "b      42f",

            // len=5: val = ((p1 & 0x07) << 32) | data_lo&0xFFFFFFFF, offset = 0x10204080
            "and    x8, x3, #0x07",
            "and    x0, x5, #0xFFFFFFFF",
            "orr    x0, x0, x8, lsl #32",
            "b      43f",

            // len=6: val = ((p1 & 0x03) << 40) | data_lo&0xFFFFFFFFFF
            "and    x8, x3, #0x03",
            "mov    x9, #0xFFFFFFFFFF",
            "and    x0, x5, x9",
            "b      44f",

            // len=7: val = ((p1 & 0x01) << 48) | data_lo&0xFFFFFFFFFFFF
            "and    x8, x3, #0x01",
            "mov    x9, #0xFFFFFFFFFFFF",
            "and    x0, x5, x9",
            "b      45f",

            // len=8: val = data_lo & 0xFFFFFFFFFFFFFF
            "mov    x9, #0xFFFFFFFFFFFFFF",
            "and    x0, x5, x9",
            "mov    x1, #0",
            "b      46f",

            // len=9: val = (p2 << 56) | (data_lo & 0x00FFFFFFFFFFFFFF)
            "mov    x9, #0x00FFFFFFFFFFFFFF",
            "and    x0, x5, x9",
            "orr    x0, x0, x4, lsl #56",
            "b      47f",

            // len=10: val = ((p2 & 0x3F) << 64) | data_lo
            "and    x8, x4, #0x3F",
            "mov    x0, x5",
            "mov    x1, x8",
            "b      48f",

            // len=11: val = ((p2 & 0x1F) << 72) | (data_hi << 64) | data_lo
            "and    x8, x4, #0x1F",
            "mov    x0, x5",
            "and    x1, x6, #0x1FFF",
            "b      49f",

            // len=12
            "and    x8, x4, #0x0F",
            "mov    x0, x5",
            "mov    x9, #0xFFFFF",
            "and    x1, x6, x9",
            "b      60f",

            // len=13
            "and    x8, x4, #0x07",
            "mov    x0, x5",
            "mov    x9, #0x7FFFFFF",
            "and    x1, x6, x9",
            "b      61f",

            // len=14
            "and    x8, x4, #0x03",
            "mov    x0, x5",
            "mov    x9, #0x3FFFFFFFF",
            "and    x1, x6, x9",
            "b      62f",

            // len=15
            "and    x8, x4, #0x01",
            "mov    x0, x5",
            "mov    x9, #0x1FFFFFFFFFF",
            "and    x1, x6, x9",
            "b      63f",

            // len=16: val = data
            "mov    x0, x5",
            "mov    x1, x6",
            "b      64f",
            "nop",

            // Offset additions for lengths 2-16
            "40:",  // len=2: + 128
            "add    x0, x0, #128",
            "mov    x1, #0",
            "b      100f",

            "41:",  // len=3: + 0x4080
            "mov    x9, #0x4080",
            "add    x0, x0, x9",
            "mov    x1, #0",
            "b      100f",

            "42:",  // len=4: + 0x204080
            "mov    x9, #0x4080",
            "movk   x9, #0x20, lsl #16",
            "add    x0, x0, x9",
            "mov    x1, #0",
            "b      100f",

            "43:",  // len=5: + 0x10204080
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "add    x0, x0, x9",
            "mov    x1, #0",
            "b      100f",

            "44:",  // len=6: finish shift + offset
            "orr    x0, x0, x8, lsl #40",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x8, lsl #32",
            "add    x0, x0, x9",
            "mov    x1, #0",
            "b      100f",

            "45:",  // len=7: finish shift + offset
            "orr    x0, x0, x8, lsl #48",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x408, lsl #32",
            "add    x0, x0, x9",
            "mov    x1, #0",
            "b      100f",

            "46:",  // len=8: + offset
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0002, lsl #48",
            "add    x0, x0, x9",
            "b      100f",

            "47:",  // len=9: + offset!(9)
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "adds   x0, x0, x9",
            "adc    x1, xzr, xzr",
            "b      100f",

            "48:",  // len=10: + offset!(10)
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "adds   x0, x0, x9",
            "mov    x8, #1",
            "adc    x1, x1, x8",
            "b      100f",

            "49:",  // len=11: + offset!(11)
            "orr    x1, x1, x8, lsl #8",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "adds   x0, x0, x9",
            "mov    x8, #65",
            "adc    x1, x1, x8",
            "b      100f",

            "60:",  // len=12: + offset!(12)
            "orr    x1, x1, x8, lsl #16",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "adds   x0, x0, x9",
            "mov    x8, #8256",
            "movk   x8, #0x01, lsl #16",
            "adc    x1, x1, x8",
            "b      100f",

            "61:",  // len=13
            "orr    x1, x1, x8, lsl #24",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "adds   x0, x0, x9",
            "mov    x8, #0x4000",
            "movk   x8, #0x0108, lsl #16",
            "adc    x1, x1, x8",
            "b      100f",

            "62:",  // len=14
            "orr    x1, x1, x8, lsl #32",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "adds   x0, x0, x9",
            "mov    x8, #0x4000",
            "movk   x8, #0x8408, lsl #16",
            "adc    x1, x1, x8",
            "b      100f",

            "63:",  // len=15
            "orr    x1, x1, x8, lsl #40",
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "adds   x0, x0, x9",
            "mov    x8, #0x4000",
            "movk   x8, #0x4208, lsl #16",
            "movk   x8, #0x04, lsl #32",
            "adc    x1, x1, x8",
            "b      100f",

            "64:",  // len=16
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "movk   x9, #0x0408, lsl #32",
            "movk   x9, #0x0102, lsl #48",
            "adds   x0, x0, x9",
            "mov    x8, #0x4000",
            "movk   x8, #0x2108, lsl #16",
            "movk   x8, #0x0204, lsl #32",
            "adc    x1, x1, x8",

            "100:",

            in("w3") p1 as u32,
            in("w4") p2 as u32,
            in("x5") data_lo,
            in("x6") data_hi,
            out("w7") _,
            out("x8") _,
            out("x9") _,
            out("x10") _,
            out("w11") _,
            out("x0") result_lo,
            out("x1") result_hi,
            options(pure, nomem, nostack),
        );
    }
    ((result_hi as u128) << 64) | (result_lo as u128)
}

/// Decode a VLQ back to u128.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_vu128(n: Vu128) -> u128 {
    decode_vu128_asm(n.0, n.1, n.2)
}

/// Decode a VLQ back to u128 using x86_64 asm.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn decode_vu128_asm_x86(p1: u8, p2: u8, data: u128) -> u128 {
    let data_lo = data as u64;
    let data_hi = (data >> 64) as u64;
    let result_lo: u64;
    let result_hi: u64;

    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Compute length from first byte
            "movzx  {len:e}, {p1:l}",
            "lzcnt  {len:e}, {len:e}",
            "sub    {len:e}, 23",

            // Check if extended prefix (p1 == 0)
            "test   {p1:l}, {p1:l}",
            "jnz    10f",

            // Extended prefix
            "cmp    {p2:l}, 0x80",
            "jae    11f",
            "test   {p2:l}, {p2:l}",
            "jz     12f",

            // len = 9 + leading_zeros(p2)
            "movzx  {tmp:e}, {p2:l}",
            "lzcnt  {tmp:e}, {tmp:e}",
            "sub    {tmp:e}, 23",
            "add    {len:e}, {tmp:e}",
            "jmp    20f",

            "10:",  // Normal prefix
            "jmp    20f",

            "11:",  // len = 9
            "mov    {len:e}, 9",
            "jmp    20f",

            "12:",  // len = 18 (raw)
            "mov    {r_lo:r}, {d_lo:r}",
            "mov    {r_hi:r}, {d_hi:r}",
            "jmp    100f",

            "20:",
            // Jump table
            "lea    {jump:r}, [rip + 30f]",
            "mov    {idx:e}, {len:e}",
            "sub    {idx:e}, 1",
            "imul   {idx:e}, {idx:e}, 24",
            "add    {jump:r}, {idx:r}",
            "jmp    {jump:r}",

            // len=1
            "30:",
            "movzx  {r_lo:e}, {p1:l}",
            "and    {r_lo:e}, 0x7F",
            "xor    {r_hi:r}, {r_hi:r}",
            "jmp    100f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=2
            "movzx  {tmp:e}, {p1:l}",
            "and    {tmp:e}, 0x3F",
            "shl    {tmp:r}, 8",
            "mov    {r_lo:r}, {d_lo:r}",
            "and    {r_lo:r}, 0xFF",
            "or     {r_lo:r}, {tmp:r}",
            "jmp    40f",

            // len=3
            "movzx  {tmp:e}, {p1:l}",
            "and    {tmp:e}, 0x1F",
            "shl    {tmp:r}, 16",
            "mov    {r_lo:r}, {d_lo:r}",
            "and    {r_lo:r}, 0xFFFF",
            "or     {r_lo:r}, {tmp:r}",
            "jmp    41f",

            // len=4
            "movzx  {tmp:e}, {p1:l}",
            "and    {tmp:e}, 0x0F",
            "shl    {tmp:r}, 24",
            "mov    {r_lo:r}, {d_lo:r}",
            "and    {r_lo:r}, 0xFFFFFF",
            "or     {r_lo:r}, {tmp:r}",
            "jmp    42f",

            // len=5
            "movzx  {tmp:e}, {p1:l}",
            "and    {tmp:e}, 0x07",
            "shl    {tmp:r}, 32",
            "mov    {r_lo:e}, {d_lo:e}",
            "or     {r_lo:r}, {tmp:r}",
            "jmp    43f",
            ".byte 0x90, 0x90, 0x90",

            // len=6
            "movzx  {tmp:e}, {p1:l}",
            "and    {tmp:e}, 0x03",
            "shl    {tmp:r}, 40",
            "movabs {r_lo:r}, 0xFFFFFFFFFF",
            "and    {r_lo:r}, {d_lo:r}",
            "or     {r_lo:r}, {tmp:r}",
            "jmp    44f",

            // len=7
            "movzx  {tmp:e}, {p1:l}",
            "and    {tmp:e}, 0x01",
            "shl    {tmp:r}, 48",
            "movabs {r_lo:r}, 0xFFFFFFFFFFFF",
            "and    {r_lo:r}, {d_lo:r}",
            "or     {r_lo:r}, {tmp:r}",
            "jmp    45f",

            // len=8
            "movabs {r_lo:r}, 0xFFFFFFFFFFFFFF",
            "and    {r_lo:r}, {d_lo:r}",
            "xor    {r_hi:r}, {r_hi:r}",
            "jmp    46f",
            ".byte 0x90, 0x90, 0x90, 0x90",

            // len=9
            "movabs {r_lo:r}, 0x00FFFFFFFFFFFFFF",
            "and    {r_lo:r}, {d_lo:r}",
            "movzx  {tmp:e}, {p2:l}",
            "shl    {tmp:r}, 56",
            "or     {r_lo:r}, {tmp:r}",
            "jmp    47f",

            // len=10
            "mov    {r_lo:r}, {d_lo:r}",
            "movzx  {r_hi:e}, {p2:l}",
            "and    {r_hi:r}, 0x3F",
            "jmp    48f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // len=11
            "mov    {r_lo:r}, {d_lo:r}",
            "mov    {r_hi:r}, {d_hi:r}",
            "and    {r_hi:r}, 0x1FFF",
            "movzx  {tmp:e}, {p2:l}",
            "and    {tmp:e}, 0x1F",
            "jmp    49f",

            // len=12
            "mov    {r_lo:r}, {d_lo:r}",
            "mov    {r_hi:r}, {d_hi:r}",
            "mov    {tmp:e}, 0xFFFFF",
            "and    {r_hi:r}, {tmp:r}",
            "movzx  {tmp:e}, {p2:l}",
            "and    {tmp:e}, 0x0F",
            "jmp    4Af",

            // len=13
            "mov    {r_lo:r}, {d_lo:r}",
            "mov    {r_hi:r}, {d_hi:r}",
            "mov    {tmp:e}, 0x7FFFFFF",
            "and    {r_hi:r}, {tmp:r}",
            "movzx  {tmp:e}, {p2:l}",
            "and    {tmp:e}, 0x07",
            "jmp    4Bf",

            // len=14
            "mov    {r_lo:r}, {d_lo:r}",
            "mov    {r_hi:r}, {d_hi:r}",
            "movabs {tmp:r}, 0x3FFFFFFFF",
            "and    {r_hi:r}, {tmp:r}",
            "movzx  {tmp:e}, {p2:l}",
            "and    {tmp:e}, 0x03",
            "jmp    4Cf",

            // len=15
            "mov    {r_lo:r}, {d_lo:r}",
            "mov    {r_hi:r}, {d_hi:r}",
            "movabs {tmp:r}, 0x1FFFFFFFFFF",
            "and    {r_hi:r}, {tmp:r}",
            "movzx  {tmp:e}, {p2:l}",
            "and    {tmp:e}, 0x01",
            "jmp    4Df",

            // len=16
            "mov    {r_lo:r}, {d_lo:r}",
            "mov    {r_hi:r}, {d_hi:r}",
            "jmp    4Ef",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",

            // Offset additions
            "40:",  // len=2: + 128
            "add    {r_lo:r}, 128",
            "xor    {r_hi:r}, {r_hi:r}",
            "jmp    100f",

            "41:",  // len=3: + 0x4080
            "add    {r_lo:r}, 0x4080",
            "xor    {r_hi:r}, {r_hi:r}",
            "jmp    100f",

            "42:",  // len=4: + 0x204080
            "add    {r_lo:r}, 0x204080",
            "xor    {r_hi:r}, {r_hi:r}",
            "jmp    100f",

            "43:",  // len=5: + 0x10204080
            "add    {r_lo:r}, 0x10204080",
            "xor    {r_hi:r}, {r_hi:r}",
            "jmp    100f",

            "44:",  // len=6
            "movabs {tmp:r}, 0x0008_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "xor    {r_hi:r}, {r_hi:r}",
            "jmp    100f",

            "45:",  // len=7
            "movabs {tmp:r}, 0x0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "xor    {r_hi:r}, {r_hi:r}",
            "jmp    100f",

            "46:",  // len=8
            "movabs {tmp:r}, 0x0002_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "jmp    100f",

            "47:",  // len=9
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "mov    {r_hi:r}, 0",
            "adc    {r_hi:r}, 0",
            "jmp    100f",

            "48:",  // len=10
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "adc    {r_hi:r}, 1",
            "jmp    100f",

            "49:",  // len=11
            "shl    {tmp:r}, 8",
            "or     {r_hi:r}, {tmp:r}",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "adc    {r_hi:r}, 65",
            "jmp    100f",

            "4Af:",  // len=12
            "shl    {tmp:r}, 16",
            "or     {r_hi:r}, {tmp:r}",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "mov    {tmp:r}, 0x00018200",
            "adc    {r_hi:r}, {tmp:r}",
            "jmp    100f",

            "4Bf:",  // len=13
            "shl    {tmp:r}, 24",
            "or     {r_hi:r}, {tmp:r}",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "mov    {tmp:r}, 0x01084000",
            "adc    {r_hi:r}, {tmp:r}",
            "jmp    100f",

            "4Cf:",  // len=14
            "shl    {tmp:r}, 32",
            "or     {r_hi:r}, {tmp:r}",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "mov    {tmp:r}, 0x84084000",
            "adc    {r_hi:r}, {tmp:r}",
            "jmp    100f",

            "4Df:",  // len=15
            "shl    {tmp:r}, 40",
            "or     {r_hi:r}, {tmp:r}",
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "movabs {tmp:r}, 0x0004_4208_4000",
            "adc    {r_hi:r}, {tmp:r}",
            "jmp    100f",

            "4Ef:",  // len=16
            "movabs {tmp:r}, 0x0102_0408_1020_4080",
            "add    {r_lo:r}, {tmp:r}",
            "movabs {tmp:r}, 0x0204_2108_4000",
            "adc    {r_hi:r}, {tmp:r}",

            "100:",

            p1 = in(reg_byte) p1,
            p2 = in(reg_byte) p2,
            d_lo = in(reg) data_lo,
            d_hi = in(reg) data_hi,
            len = out(reg) _,
            tmp = out(reg) _,
            jump = out(reg) _,
            idx = out(reg) _,
            r_lo = out(reg) result_lo,
            r_hi = out(reg) result_hi,
            options(pure, nomem, nostack),
        );
    }
    ((result_hi as u128) << 64) | (result_lo as u128)
}

/// Decode a VLQ back to u128.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn decode_vu128(n: Vu128) -> u128 {
    decode_vu128_asm_x86(n.0, n.1, n.2)
}

/// Decode a VLQ back to u128.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
#[inline(always)]
pub const fn decode_vu128(n: Vu128) -> u128 {
    let len = n.len();
    let p1 = n.0;
    let p2 = n.1;
    let data = n.2;

    match len {
        1 => (p1 & 0x7F) as u128,
        2 => ((((p1 & 0x3F) as u128) << 8) | (data & 0xFF)) + offset!(2) as u128,
        3 => ((((p1 & 0x1F) as u128) << 16) | (data & 0xFFFF)) + offset!(3) as u128,
        4 => ((((p1 & 0x0F) as u128) << 24) | (data & 0xFF_FFFF)) + offset!(4) as u128,
        5 => ((((p1 & 0x07) as u128) << 32) | (data & 0xFFFF_FFFF)) + offset!(5) as u128,
        6 => ((((p1 & 0x03) as u128) << 40) | (data & 0xFF_FFFF_FFFF)) + offset!(6) as u128,
        7 => ((((p1 & 0x01) as u128) << 48) | (data & 0xFFFF_FFFF_FFFF)) + offset!(7) as u128,
        8 => (data & 0xFF_FFFF_FFFF_FFFF) + offset!(8) as u128,
        9 => {
            let high = (p2 as u128) << 56;
            let low = data & 0x00FF_FFFF_FFFF_FFFF;
            (high | low) + offset!(9) as u128
        }
        10 => ((((p2 & 0x3F) as u128) << 64) | data) + offset!(10),
        11 => ((((p2 & 0x1F) as u128) << 72) | data) + offset!(11),
        12 => ((((p2 & 0x0F) as u128) << 80) | data) + offset!(12),
        13 => ((((p2 & 0x07) as u128) << 88) | data) + offset!(13),
        14 => ((((p2 & 0x03) as u128) << 96) | data) + offset!(14),
        15 => ((((p2 & 0x01) as u128) << 104) | data) + offset!(15),
        16 => data + offset!(16),
        _ => data, // len=18: raw
    }
}

/// Decode a u128 from a byte slice.
///
/// Returns `Some((value, bytes_consumed))` on success, or `None` if the slice is too short.
#[inline(always)]
pub fn decode_vu128_slice(data: &[u8]) -> Option<(u128, usize)> {
    let p1 = *data.first()?;

    // Fast path: 1-byte encoding (high bit set)
    if p1 & 0x80 != 0 {
        return Some(((p1 & 0x7F) as u128, 1));
    }

    // Fast path: 2-byte encoding (bit 6 set)
    if p1 & 0x40 != 0 {
        let second = *data.get(1)?;
        let val = (((p1 & 0x3F) as u128) << 8) | (second as u128);
        return Some((val + 128, 2));
    }

    // Fast path: 3-byte encoding (bit 5 set)
    if p1 & 0x20 != 0 {
        if data.len() < 3 {
            return None;
        }
        let low = u16::from_le_bytes([data[1], data[2]]) as u128;
        let val = (((p1 & 0x1F) as u128) << 16) | low;
        return Some((val + 16512, 3));
    }

    if p1 == 0 {
        // Extended format (len 9+) - need second byte for length
        let p2 = *data.get(1)?;
        let len = decode_len_vu128(p1, p2) as usize;
        if data.len() < len {
            return None;
        }
        let mut data_buf = [0u8; 16];
        if len > 2 {
            data_buf[..(len - 2)].copy_from_slice(&data[2..len]);
        }
        let packed = u128::from_le_bytes(data_buf);
        Some((decode_vu128(Vu128(p1, p2, packed)), len))
    } else {
        // Standard format (len 1-8) - data goes in self.2
        let len = decode_len_vu128(p1, 0) as usize;
        if data.len() < len {
            return None;
        }
        let mut data_buf = [0u8; 16];
        if len > 1 {
            data_buf[..(len - 1)].copy_from_slice(&data[1..len]);
        }
        let packed = u128::from_le_bytes(data_buf);
        Some((decode_vu128(Vu128(p1, 0, packed)), len))
    }
}

/// An unsigned 128-bit integer in variable-length quantity encoding.
///
/// Stored as (prefix1, prefix2, data) where:
/// - prefix1: first byte (0x00 for extended encoding)
/// - prefix2: second byte (determines extended length when prefix1=0)
/// - data: up to 128 bits of payload
#[derive(Clone, Copy)]
pub struct Vu128(pub(crate) u8, pub(crate) u8, pub(crate) u128);

#[allow(clippy::len_without_is_empty)]
impl Vu128 {
    /// Construct a new VLQ instance from the given `u128`.
    #[inline(always)]
    pub fn new(value: u128) -> Vu128 {
        encode_vu128(value)
    }

    /// Retrieve the stored number as `u128`.
    #[inline(always)]
    pub fn get(&self) -> u128 {
        decode_vu128(*self)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu128(self.0, self.1)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; VU128_BUF_SIZE] {
        let mut out = [0u8; VU128_BUF_SIZE];
        let len = self.len() as usize;
        out[0] = self.0;

        if len == 1 {
            return out;
        }

        if len <= 8 {
            // Standard format (len 2-8): data bytes from self.2, p2 unused
            let data = self.2.to_le_bytes();
            let mut i = 0;
            while i < len - 1 && i < 16 {
                out[i + 1] = data[i];
                i += 1;
            }
        } else {
            // Extended format (len 9+): p2 is significant
            out[1] = self.1;
            if len > 2 {
                let data = self.2.to_le_bytes();
                let data_bytes = len - 2;
                let mut i = 0;
                while i < data_bytes && i < 16 {
                    out[i + 2] = data[i];
                    i += 1;
                }
            }
        }

        out
    }
}

impl From<u128> for Vu128 {
    fn from(n: u128) -> Self {
        encode_vu128(n)
    }
}

impl From<Vu128> for u128 {
    fn from(n: Vu128) -> Self {
        decode_vu128(n)
    }
}

impl Display for Vu128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&u128::from(*self), f)
    }
}

impl Debug for Vu128 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vu128(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
