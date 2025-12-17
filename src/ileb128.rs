//! Signed LEB128 encoding (for benchmarking comparison).

pub const ILEB128_I32_BUF_SIZE: usize = 5;
pub const ILEB128_I64_BUF_SIZE: usize = 10;

/// Encode an i32 as ILEB128 (signed LEB128).
///
/// Returns the number of bytes written.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn encode_ileb128_i32(value: i32, buf: &mut [u8; ILEB128_I32_BUF_SIZE]) -> usize {
    let len: usize;
    let ptr = buf.as_mut_ptr();
    // SAFETY: We write at most 5 bytes, buf is exactly 5 bytes.
    unsafe {
        core::arch::asm!(
            // Comparison chain for signed LEB128 (no loop!)
            // ILEB128 ranges: len=1: -64..63, len=2: -8192..8191, etc.

            // len=1: -64 <= x <= 63
            "cmn    w0, #64",           // cmp w0, -64
            "b.lt   200f",              // if x < -64, need more bytes
            "cmp    w0, #63",
            "b.gt   200f",              // if x > 63, need more bytes
            "and    w3, w0, #0x7F",
            "strb   w3, [x1]",
            "mov    w2, #1",
            "b      900f",

            // len=2: -8192 <= x <= 8191
            "200:",
            "mov    w4, #8192",
            "neg    w5, w4",            // w5 = -8192
            "cmp    w0, w5",
            "b.lt   300f",
            "sub    w4, w4, #1",        // w4 = 8191
            "cmp    w0, w4",
            "b.gt   300f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #1]",
            "mov    w2, #2",
            "b      900f",

            // len=3: -1048576 <= x <= 1048575
            "300:",
            "mov    w4, #0x100000",     // 1048576
            "neg    w5, w4",
            "cmp    w0, w5",
            "b.lt   400f",
            "sub    w4, w4, #1",
            "cmp    w0, w4",
            "b.gt   400f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #2]",
            "mov    w2, #3",
            "b      900f",

            // len=4: -134217728 <= x <= 134217727
            "400:",
            "mov    w4, #0x8000000",    // 134217728
            "neg    w5, w4",
            "cmp    w0, w5",
            "b.lt   500f",
            "sub    w4, w4, #1",
            "cmp    w0, w4",
            "b.gt   500f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #3]",
            "mov    w2, #4",
            "b      900f",

            // len=5: everything else
            "500:",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #3]",
            "asr    w0, w0, #7",
            "and    w3, w0, #0x0F",     // Only 4 bits for byte 5
            "strb   w3, [x1, #4]",
            "mov    w2, #5",

            "900:",

            inout("w0") value => _,
            in("x1") ptr,
            lateout("w2") len,
            out("w3") _,
            out("w4") _,
            out("w5") _,
            options(nostack),
        );
    }
    len
}

/// Encode an i32 as ILEB128 (fallback).
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
#[inline(always)]
pub fn encode_ileb128_i32(mut value: i32, buf: &mut [u8; ILEB128_I32_BUF_SIZE]) -> usize {
    let mut i = 0;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7; // Arithmetic shift
        // Check if we're done: value is 0 or -1, and sign bit of byte matches
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        if done {
            buf[i] = byte;
            return i + 1;
        }
        buf[i] = byte | 0x80;
        i += 1;
    }
}

/// Decode an i32 from ILEB128.
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_ileb128_i32(buf: &[u8]) -> (i32, usize) {
    let buf_len = buf.len();
    if buf_len == 0 {
        return (0, 0);
    }

    let result: i32;
    let consumed: usize;
    let ptr = buf.as_ptr();
    // SAFETY: We check bounds before each byte read.
    unsafe {
        core::arch::asm!(
            // Byte 0 (shift 0)
            "cmp    w9, #1",
            "b.lo   300f",
            "ldrb   w3, [x1]",
            "and    w0, w3, #0x7F",
            "tbz    w3, #7, 200f",

            // Byte 1 (shift 7)
            "cmp    w9, #2",
            "b.lo   300f",
            "ldrb   w3, [x1, #1]",
            "and    w4, w3, #0x7F",
            "orr    w0, w0, w4, lsl #7",
            "tbz    w3, #7, 210f",

            // Byte 2 (shift 14)
            "cmp    w9, #3",
            "b.lo   300f",
            "ldrb   w3, [x1, #2]",
            "and    w4, w3, #0x7F",
            "orr    w0, w0, w4, lsl #14",
            "tbz    w3, #7, 220f",

            // Byte 3 (shift 21)
            "cmp    w9, #4",
            "b.lo   300f",
            "ldrb   w3, [x1, #3]",
            "and    w4, w3, #0x7F",
            "orr    w0, w0, w4, lsl #21",
            "tbz    w3, #7, 230f",

            // Byte 4 (shift 28) - final byte, only 4 bits valid
            "cmp    w9, #5",
            "b.lo   300f",
            "ldrb   w3, [x1, #4]",
            "and    w4, w3, #0x0F",
            "orr    w0, w0, w4, lsl #28",
            "tbnz   w3, #7, 300f",        // error if continuation set
            "mov    w2, #5",
            "b      400f",                // no sign extension needed (all 32 bits filled)

            // Exit points with sign extension check
            // After byte 0 (shift 7): sign extend from bit 6
            "200:",
            "mov    w2, #1",
            "tbz    w3, #6, 400f",
            "orr    w0, w0, #0xFFFFFF80",
            "b      400f",

            // After byte 1 (shift 14): sign extend from bit 13
            "210:",
            "mov    w2, #2",
            "tbz    w3, #6, 400f",
            "orr    w0, w0, #0xFFFFC000",
            "b      400f",

            // After byte 2 (shift 21): sign extend from bit 20
            "220:",
            "mov    w2, #3",
            "tbz    w3, #6, 400f",
            "orr    w0, w0, #0xFFE00000",
            "b      400f",

            // After byte 3 (shift 28): sign extend from bit 27
            "230:",
            "mov    w2, #4",
            "tbz    w3, #6, 400f",
            "orr    w0, w0, #0xF0000000",
            "b      400f",

            "300:",                       // error
            "mov    w0, #0",
            "mov    w2, #0",

            "400:",                       // final exit

            in("x1") ptr,
            in("x9") buf_len,
            lateout("w0") result,
            lateout("w2") consumed,
            out("w3") _,
            out("w4") _,
            options(readonly, nostack),
        );
    }
    (result, consumed)
}

/// Decode an i32 from ILEB128 (fallback).
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
#[inline(always)]
pub fn decode_ileb128_i32(buf: &[u8]) -> (i32, usize) {
    let mut result: i32 = 0;
    let mut shift = 0;
    let mut last_byte = 0u8;
    for (i, &byte) in buf.iter().enumerate() {
        if i >= 5 {
            return (0, 0);
        }
        result |= ((byte & 0x7f) as i32) << shift;
        shift += 7;
        last_byte = byte;
        if byte & 0x80 == 0 {
            // Sign extend if necessary
            if shift < 32 && (last_byte & 0x40) != 0 {
                result |= !0 << shift;
            }
            return (result, i + 1);
        }
    }
    (0, 0)
}

/// Encode an i64 as ILEB128 (signed LEB128).
///
/// Returns the number of bytes written.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn encode_ileb128_i64(value: i64, buf: &mut [u8; ILEB128_I64_BUF_SIZE]) -> usize {
    let len: usize;
    let ptr = buf.as_mut_ptr();
    // SAFETY: We write at most 10 bytes, buf is exactly 10 bytes.
    unsafe {
        core::arch::asm!(
            // Comparison chain for signed LEB128 (no loop!)
            // Ranges: len=1: -64..63, len=2: -8192..8191, etc.

            // len=1: -64 <= x <= 63
            "cmn    x0, #64",
            "b.lt   200f",
            "cmp    x0, #63",
            "b.gt   200f",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1]",
            "mov    w2, #1",
            "b      9000f",

            // len=2: -8192 <= x <= 8191
            "200:",
            "mov    x4, #8192",
            "neg    x5, x4",
            "cmp    x0, x5",
            "b.lt   300f",
            "sub    x4, x4, #1",
            "cmp    x0, x4",
            "b.gt   300f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #1]",
            "mov    w2, #2",
            "b      9000f",

            // len=3: -1048576 <= x <= 1048575
            "300:",
            "mov    x4, #0x100000",
            "neg    x5, x4",
            "cmp    x0, x5",
            "b.lt   400f",
            "sub    x4, x4, #1",
            "cmp    x0, x4",
            "b.gt   400f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #2]",
            "mov    w2, #3",
            "b      9000f",

            // len=4: -134217728 <= x <= 134217727
            "400:",
            "mov    x4, #0x8000000",
            "neg    x5, x4",
            "cmp    x0, x5",
            "b.lt   500f",
            "sub    x4, x4, #1",
            "cmp    x0, x4",
            "b.gt   500f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #3]",
            "mov    w2, #4",
            "b      9000f",

            // len=5: -17179869184 <= x <= 17179869183
            "500:",
            "mov    x4, #0x400000000",
            "neg    x5, x4",
            "cmp    x0, x5",
            "b.lt   600f",
            "sub    x4, x4, #1",
            "cmp    x0, x4",
            "b.gt   600f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #3]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #4]",
            "mov    w2, #5",
            "b      9000f",

            // len=6: -2199023255552 <= x <= 2199023255551
            "600:",
            "mov    x4, #0x20000000000",
            "neg    x5, x4",
            "cmp    x0, x5",
            "b.lt   700f",
            "sub    x4, x4, #1",
            "cmp    x0, x4",
            "b.gt   700f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #3]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #4]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #5]",
            "mov    w2, #6",
            "b      9000f",

            // len=7: -281474976710656 <= x <= 281474976710655
            "700:",
            "mov    x4, #0x1000000000000",
            "neg    x5, x4",
            "cmp    x0, x5",
            "b.lt   800f",
            "sub    x4, x4, #1",
            "cmp    x0, x4",
            "b.gt   800f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #3]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #4]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #5]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #6]",
            "mov    w2, #7",
            "b      9000f",

            // len=8: -36028797018963968 <= x <= 36028797018963967
            "800:",
            "mov    x4, #0x80000000000000",
            "neg    x5, x4",
            "cmp    x0, x5",
            "b.lt   900f",
            "sub    x4, x4, #1",
            "cmp    x0, x4",
            "b.gt   900f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #3]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #4]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #5]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #6]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #7]",
            "mov    w2, #8",
            "b      9000f",

            // len=9: -4611686018427387904 <= x <= 4611686018427387903
            "900:",
            "mov    x4, #0x4000000000000000",
            "neg    x5, x4",
            "cmp    x0, x5",
            "b.lt   1000f",
            "sub    x4, x4, #1",
            "cmp    x0, x4",
            "b.gt   1000f",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #3]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #4]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #5]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #6]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #7]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "strb   w3, [x1, #8]",
            "mov    w2, #9",
            "b      9000f",

            // len=10: everything else
            "1000:",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #1]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #2]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #3]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #4]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #5]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #6]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #7]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x7F",
            "orr    w3, w3, #0x80",
            "strb   w3, [x1, #8]",
            "asr    x0, x0, #7",
            "and    w3, w0, #0x01",     // Only 1 bit for byte 10
            "strb   w3, [x1, #9]",
            "mov    w2, #10",

            "9000:",

            inout("x0") value => _,
            in("x1") ptr,
            lateout("w2") len,
            out("w3") _,
            out("x4") _,
            out("x5") _,
            options(nostack),
        );
    }
    len
}

/// Encode an i64 as ILEB128 (fallback).
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
#[inline(always)]
pub fn encode_ileb128_i64(mut value: i64, buf: &mut [u8; ILEB128_I64_BUF_SIZE]) -> usize {
    let mut i = 0;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7; // Arithmetic shift
        // Check if we're done: value is 0 or -1, and sign bit of byte matches
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        if done {
            buf[i] = byte;
            return i + 1;
        }
        buf[i] = byte | 0x80;
        i += 1;
    }
}

/// Decode an i64 from ILEB128.
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_ileb128_i64(buf: &[u8]) -> (i64, usize) {
    let buf_len = buf.len();
    if buf_len == 0 {
        return (0, 0);
    }

    let result: i64;
    let consumed: usize;
    let ptr = buf.as_ptr();
    // SAFETY: We check bounds before each byte read.
    unsafe {
        core::arch::asm!(
            // Byte 0 (shift 0)
            "cmp    w9, #1",
            "b.lo   300f",
            "ldrb   w3, [x1]",
            "and    x0, x3, #0x7F",
            "tbz    w3, #7, 200f",

            // Byte 1 (shift 7)
            "cmp    w9, #2",
            "b.lo   300f",
            "ldrb   w3, [x1, #1]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #7",
            "tbz    w3, #7, 201f",

            // Byte 2 (shift 14)
            "cmp    w9, #3",
            "b.lo   300f",
            "ldrb   w3, [x1, #2]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #14",
            "tbz    w3, #7, 202f",

            // Byte 3 (shift 21)
            "cmp    w9, #4",
            "b.lo   300f",
            "ldrb   w3, [x1, #3]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #21",
            "tbz    w3, #7, 203f",

            // Byte 4 (shift 28)
            "cmp    w9, #5",
            "b.lo   300f",
            "ldrb   w3, [x1, #4]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #28",
            "tbz    w3, #7, 204f",

            // Byte 5 (shift 35)
            "cmp    w9, #6",
            "b.lo   300f",
            "ldrb   w3, [x1, #5]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #35",
            "tbz    w3, #7, 205f",

            // Byte 6 (shift 42)
            "cmp    w9, #7",
            "b.lo   300f",
            "ldrb   w3, [x1, #6]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #42",
            "tbz    w3, #7, 206f",

            // Byte 7 (shift 49)
            "cmp    w9, #8",
            "b.lo   300f",
            "ldrb   w3, [x1, #7]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #49",
            "tbz    w3, #7, 207f",

            // Byte 8 (shift 56)
            "cmp    w9, #9",
            "b.lo   300f",
            "ldrb   w3, [x1, #8]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #56",
            "tbz    w3, #7, 208f",

            // Byte 9 (shift 63) - final byte, only 1 bit valid
            "cmp    w9, #10",
            "b.lo   300f",
            "ldrb   w3, [x1, #9]",
            "and    x4, x3, #0x01",
            "orr    x0, x0, x4, lsl #63",
            "tbnz   w3, #7, 300f",        // error if continuation set
            "mov    w2, #10",
            "b      400f",                // no sign extension needed (all 64 bits filled)

            // Exit points with sign extension check
            // After byte 0 (shift 7): sign extend from bit 6
            "200:",
            "mov    w2, #1",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFFFFFFFFF80",
            "b      400f",

            // After byte 1 (shift 14)
            "201:",
            "mov    w2, #2",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFFFFFFFC000",
            "b      400f",

            // After byte 2 (shift 21)
            "202:",
            "mov    w2, #3",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFFFFFE00000",
            "b      400f",

            // After byte 3 (shift 28)
            "203:",
            "mov    w2, #4",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFFFF0000000",
            "b      400f",

            // After byte 4 (shift 35)
            "204:",
            "mov    w2, #5",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFF800000000",
            "b      400f",

            // After byte 5 (shift 42)
            "205:",
            "mov    w2, #6",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFC0000000000",
            "b      400f",

            // After byte 6 (shift 49)
            "206:",
            "mov    w2, #7",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFE000000000000",
            "b      400f",

            // After byte 7 (shift 56)
            "207:",
            "mov    w2, #8",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFF00000000000000",
            "b      400f",

            // After byte 8 (shift 63)
            "208:",
            "mov    w2, #9",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0x8000000000000000",
            "b      400f",

            "300:",                       // error
            "mov    x0, #0",
            "mov    w2, #0",

            "400:",                       // final exit

            in("x1") ptr,
            in("x9") buf_len,
            lateout("x0") result,
            lateout("w2") consumed,
            out("w3") _,
            out("x4") _,
            options(readonly, nostack),
        );
    }
    (result, consumed)
}

/// Decode an i64 from ILEB128 (fallback).
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
#[inline(always)]
pub fn decode_ileb128_i64(buf: &[u8]) -> (i64, usize) {
    let mut result: i64 = 0;
    let mut shift = 0;
    let mut last_byte = 0u8;
    for (i, &byte) in buf.iter().enumerate() {
        if i >= 10 {
            return (0, 0);
        }
        result |= ((byte & 0x7f) as i64) << shift;
        shift += 7;
        last_byte = byte;
        if byte & 0x80 == 0 {
            // Sign extend if necessary
            if shift < 64 && (last_byte & 0x40) != 0 {
                result |= !0i64 << shift;
            }
            return (result, i + 1);
        }
    }
    (0, 0)
}

// i128 support (19 bytes max)

pub const ILEB128_I128_BUF_SIZE: usize = 19;

/// Encode an i128 as ILEB128 (signed LEB128).
///
/// Returns the number of bytes written.
#[inline(always)]
pub fn encode_ileb128_i128(mut value: i128, buf: &mut [u8; ILEB128_I128_BUF_SIZE]) -> usize {
    let mut i = 0;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7; // Arithmetic shift
        // Check if we're done: value is 0 or -1, and sign bit of byte matches
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        if done {
            buf[i] = byte;
            return i + 1;
        }
        buf[i] = byte | 0x80;
        i += 1;
    }
}

/// Decode an i128 from ILEB128.
///
/// Returns (value, bytes_consumed). Returns (0, 0) for empty/invalid input.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_ileb128_i128(buf: &[u8]) -> (i128, usize) {
    let buf_len = buf.len();
    if buf_len == 0 {
        return (0, 0);
    }

    let result_lo: u64;
    let result_hi: u64;
    let consumed: usize;
    let ptr = buf.as_ptr();
    // SAFETY: We check bounds before each byte read.
    unsafe {
        core::arch::asm!(
            "mov    x8, #0",              // result_hi (x0 set by first byte)

            // Byte 0 (shift 0) - to low
            "cmp    w9, #1",
            "b.lo   300f",
            "ldrb   w3, [x1]",
            "and    x0, x3, #0x7F",
            "tbz    w3, #7, 200f",

            // Byte 1 (shift 7) - to low
            "cmp    w9, #2",
            "b.lo   300f",
            "ldrb   w3, [x1, #1]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #7",
            "tbz    w3, #7, 201f",

            // Byte 2 (shift 14) - to low
            "cmp    w9, #3",
            "b.lo   300f",
            "ldrb   w3, [x1, #2]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #14",
            "tbz    w3, #7, 202f",

            // Byte 3 (shift 21) - to low
            "cmp    w9, #4",
            "b.lo   300f",
            "ldrb   w3, [x1, #3]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #21",
            "tbz    w3, #7, 203f",

            // Byte 4 (shift 28) - to low
            "cmp    w9, #5",
            "b.lo   300f",
            "ldrb   w3, [x1, #4]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #28",
            "tbz    w3, #7, 204f",

            // Byte 5 (shift 35) - to low
            "cmp    w9, #6",
            "b.lo   300f",
            "ldrb   w3, [x1, #5]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #35",
            "tbz    w3, #7, 205f",

            // Byte 6 (shift 42) - to low
            "cmp    w9, #7",
            "b.lo   300f",
            "ldrb   w3, [x1, #6]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #42",
            "tbz    w3, #7, 206f",

            // Byte 7 (shift 49) - to low
            "cmp    w9, #8",
            "b.lo   300f",
            "ldrb   w3, [x1, #7]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #49",
            "tbz    w3, #7, 207f",

            // Byte 8 (shift 56) - to low
            "cmp    w9, #9",
            "b.lo   300f",
            "ldrb   w3, [x1, #8]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #56",
            "tbz    w3, #7, 208f",

            // Byte 9 (shift 63) - spans boundary: 1 bit to low, 6 bits to high
            "cmp    w9, #10",
            "b.lo   300f",
            "ldrb   w3, [x1, #9]",
            "and    x4, x3, #0x01",
            "orr    x0, x0, x4, lsl #63",
            "and    x4, x3, #0x7F",
            "lsr    x4, x4, #1",
            "orr    x8, x8, x4",
            "tbz    w3, #7, 209f",

            // Byte 10 (shift 70) - to high (shift 70-64=6)
            "cmp    w9, #11",
            "b.lo   300f",
            "ldrb   w3, [x1, #10]",
            "and    x4, x3, #0x7F",
            "orr    x8, x8, x4, lsl #6",
            "tbz    w3, #7, 210f",

            // Byte 11 (shift 77) - to high (shift 77-64=13)
            "cmp    w9, #12",
            "b.lo   300f",
            "ldrb   w3, [x1, #11]",
            "and    x4, x3, #0x7F",
            "orr    x8, x8, x4, lsl #13",
            "tbz    w3, #7, 211f",

            // Byte 12 (shift 84) - to high (shift 84-64=20)
            "cmp    w9, #13",
            "b.lo   300f",
            "ldrb   w3, [x1, #12]",
            "and    x4, x3, #0x7F",
            "orr    x8, x8, x4, lsl #20",
            "tbz    w3, #7, 212f",

            // Byte 13 (shift 91) - to high (shift 91-64=27)
            "cmp    w9, #14",
            "b.lo   300f",
            "ldrb   w3, [x1, #13]",
            "and    x4, x3, #0x7F",
            "orr    x8, x8, x4, lsl #27",
            "tbz    w3, #7, 213f",

            // Byte 14 (shift 98) - to high (shift 98-64=34)
            "cmp    w9, #15",
            "b.lo   300f",
            "ldrb   w3, [x1, #14]",
            "and    x4, x3, #0x7F",
            "orr    x8, x8, x4, lsl #34",
            "tbz    w3, #7, 214f",

            // Byte 15 (shift 105) - to high (shift 105-64=41)
            "cmp    w9, #16",
            "b.lo   300f",
            "ldrb   w3, [x1, #15]",
            "and    x4, x3, #0x7F",
            "orr    x8, x8, x4, lsl #41",
            "tbz    w3, #7, 215f",

            // Byte 16 (shift 112) - to high (shift 112-64=48)
            "cmp    w9, #17",
            "b.lo   300f",
            "ldrb   w3, [x1, #16]",
            "and    x4, x3, #0x7F",
            "orr    x8, x8, x4, lsl #48",
            "tbz    w3, #7, 216f",

            // Byte 17 (shift 119) - to high (shift 119-64=55)
            "cmp    w9, #18",
            "b.lo   300f",
            "ldrb   w3, [x1, #17]",
            "and    x4, x3, #0x7F",
            "orr    x8, x8, x4, lsl #55",
            "tbz    w3, #7, 217f",

            // Byte 18 (shift 126) - to high, only 2 bits valid (shift 126-64=62)
            "cmp    w9, #19",
            "b.lo   300f",
            "ldrb   w3, [x1, #18]",
            "and    x4, x3, #0x03",
            "orr    x8, x8, x4, lsl #62",
            "tbnz   w3, #7, 300f",        // error if continuation set
            "mov    w2, #19",
            "b      400f",                // no sign extension (all 128 bits filled)

            // Exit points with sign extension
            // Bytes 0-8: extend both low and high
            "200:",
            "mov    w2, #1",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFFFFFFFFF80",
            "mvn    x8, xzr",
            "b      400f",

            "201:",
            "mov    w2, #2",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFFFFFFFC000",
            "mvn    x8, xzr",
            "b      400f",

            "202:",
            "mov    w2, #3",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFFFFFE00000",
            "mvn    x8, xzr",
            "b      400f",

            "203:",
            "mov    w2, #4",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFFFF0000000",
            "mvn    x8, xzr",
            "b      400f",

            "204:",
            "mov    w2, #5",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFFF800000000",
            "mvn    x8, xzr",
            "b      400f",

            "205:",
            "mov    w2, #6",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFFFC0000000000",
            "mvn    x8, xzr",
            "b      400f",

            "206:",
            "mov    w2, #7",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFFFE000000000000",
            "mvn    x8, xzr",
            "b      400f",

            "207:",
            "mov    w2, #8",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0xFF00000000000000",
            "mvn    x8, xzr",
            "b      400f",

            "208:",
            "mov    w2, #9",
            "tbz    w3, #6, 400f",
            "orr    x0, x0, #0x8000000000000000",
            "mvn    x8, xzr",
            "b      400f",

            // Bytes 9-17: extend high only
            "209:",
            "mov    w2, #10",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xFFFFFFFFFFFFFFC0",
            "b      400f",

            "210:",
            "mov    w2, #11",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xFFFFFFFFFFFFE000",
            "b      400f",

            "211:",
            "mov    w2, #12",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xFFFFFFFFFFF00000",
            "b      400f",

            "212:",
            "mov    w2, #13",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xFFFFFFFFF8000000",
            "b      400f",

            "213:",
            "mov    w2, #14",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xFFFFFFFC00000000",
            "b      400f",

            "214:",
            "mov    w2, #15",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xFFFFFE0000000000",
            "b      400f",

            "215:",
            "mov    w2, #16",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xFFFF000000000000",
            "b      400f",

            "216:",
            "mov    w2, #17",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xFF80000000000000",
            "b      400f",

            "217:",
            "mov    w2, #18",
            "tbz    w3, #6, 400f",
            "orr    x8, x8, #0xC000000000000000",
            "b      400f",

            "300:",                       // error
            "mov    x0, #0",
            "mov    x8, #0",
            "mov    w2, #0",

            "400:",                       // final exit

            in("x1") ptr,
            in("x9") buf_len,
            lateout("x0") result_lo,
            lateout("x8") result_hi,
            lateout("w2") consumed,
            out("w3") _,
            out("x4") _,
            options(readonly, nostack),
        );
    }
    let value = ((result_hi as u128) << 64) | (result_lo as u128);
    (value as i128, consumed)
}

/// Decode an i128 from ILEB128 (fallback).
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
#[inline(always)]
pub fn decode_ileb128_i128(buf: &[u8]) -> (i128, usize) {
    let mut result: i128 = 0;
    let mut shift = 0;
    let mut last_byte = 0u8;
    for (i, &byte) in buf.iter().enumerate() {
        if i >= 19 {
            return (0, 0);
        }
        result |= ((byte & 0x7f) as i128) << shift;
        shift += 7;
        last_byte = byte;
        if byte & 0x80 == 0 {
            // Sign extend if necessary
            if shift < 128 && (last_byte & 0x40) != 0 {
                result |= !0i128 << shift;
            }
            return (result, i + 1);
        }
    }
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ileb128_i32_roundtrip() {
        for value in [
            0i32,
            1,
            -1,
            63,
            -64,
            64,
            -65,
            127,
            -128,
            128,
            -129,
            8191,
            -8192,
            8192,
            -8193,
            i32::MAX,
            i32::MIN,
        ] {
            let mut buf = [0u8; ILEB128_I32_BUF_SIZE];
            let len = encode_ileb128_i32(value, &mut buf);
            let (decoded, consumed) = decode_ileb128_i32(&buf);
            assert_eq!(decoded, value, "value={} len={}", value, len);
            assert_eq!(consumed, len);
        }
    }

    #[test]
    fn test_ileb128_i64_roundtrip() {
        for value in [
            0i64,
            1,
            -1,
            63,
            -64,
            64,
            -65,
            127,
            -128,
            128,
            -129,
            8191,
            -8192,
            8192,
            -8193,
            i32::MAX as i64,
            i32::MIN as i64,
            i64::MAX,
            i64::MIN,
        ] {
            let mut buf = [0u8; ILEB128_I64_BUF_SIZE];
            let len = encode_ileb128_i64(value, &mut buf);
            let (decoded, consumed) = decode_ileb128_i64(&buf);
            assert_eq!(decoded, value, "value={} len={}", value, len);
            assert_eq!(consumed, len);
        }
    }

    #[test]
    fn test_ileb128_known_encodings() {
        // Test against known LEB128 encodings
        // -1 should encode to 0x7F (single byte)
        let mut buf = [0u8; ILEB128_I64_BUF_SIZE];
        let len = encode_ileb128_i64(-1, &mut buf);
        assert_eq!(len, 1);
        assert_eq!(buf[0], 0x7F);

        // -128 should encode to 0x80 0x7F (two bytes)
        let len = encode_ileb128_i64(-128, &mut buf);
        assert_eq!(len, 2);
        assert_eq!(buf[0], 0x80);
        assert_eq!(buf[1], 0x7F);

        // 128 should encode to 0x80 0x01 (two bytes)
        let len = encode_ileb128_i64(128, &mut buf);
        assert_eq!(len, 2);
        assert_eq!(buf[0], 0x80);
        assert_eq!(buf[1], 0x01);
    }

    #[test]
    fn test_ileb128_i128_roundtrip() {
        for value in [
            0i128,
            1,
            -1,
            63,
            -64,
            64,
            -65,
            i64::MAX as i128,
            i64::MIN as i128,
            i64::MAX as i128 + 1,
            i64::MIN as i128 - 1,
            i128::MAX,
            i128::MIN,
        ] {
            let mut buf = [0u8; ILEB128_I128_BUF_SIZE];
            let len = encode_ileb128_i128(value, &mut buf);
            let (decoded, consumed) = decode_ileb128_i128(&buf);
            assert_eq!(decoded, value, "value={} len={}", value, len);
            assert_eq!(consumed, len);
        }
    }
}
