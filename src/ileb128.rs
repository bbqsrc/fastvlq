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
/// Returns (value, bytes_consumed).
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_ileb128_i32(buf: &[u8]) -> (i32, usize) {
    let result: i32;
    let consumed: usize;
    let ptr = buf.as_ptr();
    // SAFETY: Reads up to 5 bytes.
    unsafe {
        core::arch::asm!(
            "mov    w0, #0",            // result
            "mov    w2, #0",            // byte index
            "mov    w5, #0",            // shift amount

            "100:",  // loop
            "ldrb   w3, [x1, w2, uxtw]",
            "and    w4, w3, #0x7F",     // extract 7 bits
            "lsl    w4, w4, w5",        // shift into position
            "orr    w0, w0, w4",        // accumulate
            "add    w2, w2, #1",
            "add    w5, w5, #7",
            "tbnz   w3, #7, 100b",      // if continuation bit set, continue

            // Sign extend if final byte has bit 6 set and shift < 32
            "cmp    w5, #32",
            "b.ge   200f",
            "tbz    w3, #6, 200f",      // if sign bit clear, no extension needed
            // Sign extend: result |= (~0 << shift)
            "mvn    w4, wzr",           // w4 = -1
            "lsl    w4, w4, w5",        // shift
            "orr    w0, w0, w4",        // apply sign extension

            "200:",

            in("x1") ptr,
            lateout("w0") result,
            lateout("w2") consumed,
            out("w3") _,
            out("w4") _,
            out("w5") _,
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
    let mut i = 0;
    let mut byte;
    loop {
        byte = buf[i];
        result |= ((byte & 0x7f) as i32) << shift;
        i += 1;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }
    // Sign extend if necessary
    if shift < 32 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }
    (result, i)
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
/// Returns (value, bytes_consumed).
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_ileb128_i64(buf: &[u8]) -> (i64, usize) {
    let result: i64;
    let consumed: usize;
    let ptr = buf.as_ptr();
    // SAFETY: Reads up to 10 bytes.
    unsafe {
        core::arch::asm!(
            "mov    x0, #0",            // result
            "mov    w2, #0",            // byte index
            "mov    w5, #0",            // shift amount

            "100:",  // loop
            "ldrb   w3, [x1, w2, uxtw]",
            "and    x4, x3, #0x7F",     // extract 7 bits
            "lsl    x4, x4, x5",        // shift into position
            "orr    x0, x0, x4",        // accumulate
            "add    w2, w2, #1",
            "add    w5, w5, #7",
            "tbnz   w3, #7, 100b",      // if continuation bit set, continue

            // Sign extend if final byte has bit 6 set and shift < 64
            "cmp    w5, #64",
            "b.ge   200f",
            "tbz    w3, #6, 200f",      // if sign bit clear, no extension needed
            // Sign extend: result |= (~0 << shift)
            "mvn    x4, xzr",           // x4 = -1
            "lsl    x4, x4, x5",        // shift
            "orr    x0, x0, x4",        // apply sign extension

            "200:",

            in("x1") ptr,
            lateout("x0") result,
            lateout("w2") consumed,
            out("w3") _,
            out("x4") _,
            out("w5") _,
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
    let mut i = 0;
    let mut byte;
    loop {
        byte = buf[i];
        result |= ((byte & 0x7f) as i64) << shift;
        i += 1;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }
    // Sign extend if necessary
    if shift < 64 && (byte & 0x40) != 0 {
        result |= !0i64 << shift;
    }
    (result, i)
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
/// Returns (value, bytes_consumed).
#[inline(always)]
pub fn decode_ileb128_i128(buf: &[u8]) -> (i128, usize) {
    let mut result: i128 = 0;
    let mut shift = 0;
    let mut i = 0;
    let mut byte;
    loop {
        byte = buf[i];
        result |= ((byte & 0x7f) as i128) << shift;
        i += 1;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }
    // Sign extend if necessary
    if shift < 128 && (byte & 0x40) != 0 {
        result |= !0i128 << shift;
    }
    (result, i)
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
