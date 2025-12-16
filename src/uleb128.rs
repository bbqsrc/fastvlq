//! Unsigned LEB128 encoding (for benchmarking comparison).

pub const ULEB128_U32_BUF_SIZE: usize = 5;
pub const ULEB128_U64_BUF_SIZE: usize = 10;

/// Encode a u32 as ULEB128.
///
/// Returns the number of bytes written.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn encode_uleb128_u32(value: u32, buf: &mut [u8; ULEB128_U32_BUF_SIZE]) -> usize {
    let len: usize;
    let ptr = buf.as_mut_ptr();
    // SAFETY: We write at most 5 bytes, buf is exactly 5 bytes.
    unsafe {
        core::arch::asm!(
            // Calculate byte count: (32 - clz(value | 1) + 6) / 7
            "orr    w4, w0, #1",
            "clz    w4, w4",
            "mov    w5, #32",
            "sub    w4, w5, w4",        // bit_width = 32 - clz
            "add    w4, w4, #6",        // + 6 for ceiling
            "mov    w5, #7",
            "udiv   w4, w4, w5",        // byte_count = (bit_width + 6) / 7

            // Jump table by byte count (1-5), 16-byte entries
            "adr    x10, 100f",
            "sub    w11, w4, #1",
            "add    x10, x10, w11, uxtw #4",  // 16-byte entries
            "br     x10",

            // len=1 (values 0-127) - 4 instructions = 16 bytes
            "100:",
            "and    w5, w0, #0x7F",
            "strb   w5, [x1]",
            "mov    w2, #1",
            "b      200f",

            // len=2 (values 128-16383)
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1]",
            "b      102f",

            // len=3 (values 16384-2097151)
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1]",
            "b      103f",

            // len=4 (values 2097152-268435455)
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1]",
            "b      104f",

            // len=5 (values 268435456-4294967295)
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1]",
            "b      105f",

            // len=2 continuation
            "102:",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "strb   w5, [x1, #1]",
            "mov    w2, #2",
            "b      200f",

            // len=3 continuation
            "103:",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1, #1]",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "strb   w5, [x1, #2]",
            "mov    w2, #3",
            "b      200f",

            // len=4 continuation
            "104:",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1, #1]",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1, #2]",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "strb   w5, [x1, #3]",
            "mov    w2, #4",
            "b      200f",

            // len=5 continuation
            "105:",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1, #1]",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1, #2]",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "orr    w5, w5, #0x80",
            "strb   w5, [x1, #3]",
            "lsr    w0, w0, #7",
            "and    w5, w0, #0x7F",
            "strb   w5, [x1, #4]",
            "mov    w2, #5",

            "200:",

            in("w0") value,
            in("x1") ptr,
            lateout("w2") len,
            out("w4") _,
            out("w5") _,
            out("x10") _,
            out("w11") _,
            options(nostack),
        );
    }
    len
}

/// Encode a u32 as ULEB128 (fallback).
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn encode_uleb128_u32(mut value: u32, buf: &mut [u8; ULEB128_U32_BUF_SIZE]) -> usize {
    let mut i = 0;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            buf[i] = byte;
            return i + 1;
        }
        buf[i] = byte | 0x80;
        i += 1;
    }
}

/// Decode a u32 from ULEB128.
///
/// Returns (value, bytes_consumed).
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn decode_uleb128_u32(buf: &[u8]) -> (u32, usize) {
    let result: u32;
    let consumed: usize;
    let ptr = buf.as_ptr();
    // SAFETY: Reads up to 5 bytes.
    unsafe {
        core::arch::asm!(
            // Byte 1
            "ldrb   w3, [x1]",
            "and    w0, w3, #0x7F",
            "tbz    w3, #7, 101f",

            // Byte 2
            "ldrb   w3, [x1, #1]",
            "and    w4, w3, #0x7F",
            "orr    w0, w0, w4, lsl #7",
            "tbz    w3, #7, 102f",

            // Byte 3
            "ldrb   w3, [x1, #2]",
            "and    w4, w3, #0x7F",
            "orr    w0, w0, w4, lsl #14",
            "tbz    w3, #7, 103f",

            // Byte 4
            "ldrb   w3, [x1, #3]",
            "and    w4, w3, #0x7F",
            "orr    w0, w0, w4, lsl #21",
            "tbz    w3, #7, 104f",

            // Byte 5
            "ldrb   w3, [x1, #4]",
            "and    w4, w3, #0x0F",  // Only 4 bits matter for u32
            "orr    w0, w0, w4, lsl #28",
            "mov    w2, #5",
            "b      200f",

            "101:",
            "mov    w2, #1",
            "b      200f",

            "102:",
            "mov    w2, #2",
            "b      200f",

            "103:",
            "mov    w2, #3",
            "b      200f",

            "104:",
            "mov    w2, #4",

            "200:",

            in("x1") ptr,
            lateout("w0") result,
            lateout("w2") consumed,
            out("w3") _,
            out("w4") _,
            options(readonly, nostack),
        );
    }
    (result, consumed)
}

/// Decode a u32 from ULEB128 (fallback).
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn decode_uleb128_u32(buf: &[u8]) -> (u32, usize) {
    let mut result: u32 = 0;
    let mut shift = 0;
    let mut i = 0;
    loop {
        let byte = buf[i];
        result |= ((byte & 0x7f) as u32) << shift;
        i += 1;
        if byte & 0x80 == 0 {
            return (result, i);
        }
        shift += 7;
    }
}

/// Encode a u64 as ULEB128.
///
/// Returns the number of bytes written.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn encode_uleb128_u64(value: u64, buf: &mut [u8; ULEB128_U64_BUF_SIZE]) -> usize {
    let len: usize;
    let ptr = buf.as_mut_ptr();
    // SAFETY: We write at most 10 bytes, buf is exactly 10 bytes.
    unsafe {
        core::arch::asm!(
            // Optimized loop-based ULEB128 encode
            "mov    w2, #0",            // byte index

            "100:",  // loop start
            "and    w3, w0, #0x7F",     // byte = value & 0x7F
            "lsr    x0, x0, #7",        // value >>= 7

            // Check if done (value == 0)
            "cbz    x0, 200f",          // if value == 0, we're done

            // More bytes needed
            "orr    w3, w3, #0x80",     // set continuation bit
            "strb   w3, [x1, w2, uxtw]",
            "add    w2, w2, #1",
            "b      100b",

            "200:",  // done
            "strb   w3, [x1, w2, uxtw]",
            "add    w2, w2, #1",        // length = index + 1

            inout("x0") value => _,
            in("x1") ptr,
            lateout("w2") len,
            out("w3") _,
            options(nostack),
        );
    }
    len
}

/// Encode a u64 as ULEB128 (fallback).
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn encode_uleb128_u64(mut value: u64, buf: &mut [u8; ULEB128_U64_BUF_SIZE]) -> usize {
    let mut i = 0;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            buf[i] = byte;
            return i + 1;
        }
        buf[i] = byte | 0x80;
        i += 1;
    }
}

/// Decode a u64 from ULEB128.
///
/// Returns (value, bytes_consumed).
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn decode_uleb128_u64(buf: &[u8]) -> (u64, usize) {
    let result: u64;
    let consumed: usize;
    let ptr = buf.as_ptr();
    // SAFETY: Reads up to 10 bytes.
    unsafe {
        core::arch::asm!(
            // Byte 1
            "ldrb   w3, [x1]",
            "and    x0, x3, #0x7F",
            "tbz    w3, #7, 101f",

            // Byte 2
            "ldrb   w3, [x1, #1]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #7",
            "tbz    w3, #7, 102f",

            // Byte 3
            "ldrb   w3, [x1, #2]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #14",
            "tbz    w3, #7, 103f",

            // Byte 4
            "ldrb   w3, [x1, #3]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #21",
            "tbz    w3, #7, 104f",

            // Byte 5
            "ldrb   w3, [x1, #4]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #28",
            "tbz    w3, #7, 105f",

            // Byte 6
            "ldrb   w3, [x1, #5]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #35",
            "tbz    w3, #7, 106f",

            // Byte 7
            "ldrb   w3, [x1, #6]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #42",
            "tbz    w3, #7, 107f",

            // Byte 8
            "ldrb   w3, [x1, #7]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #49",
            "tbz    w3, #7, 108f",

            // Byte 9
            "ldrb   w3, [x1, #8]",
            "and    x4, x3, #0x7F",
            "orr    x0, x0, x4, lsl #56",
            "tbz    w3, #7, 109f",

            // Byte 10
            "ldrb   w3, [x1, #9]",
            "and    x4, x3, #0x01",  // Only 1 bit matters
            "orr    x0, x0, x4, lsl #63",
            "mov    w2, #10",
            "b      200f",

            "101:",
            "mov    w2, #1",
            "b      200f",

            "102:",
            "mov    w2, #2",
            "b      200f",

            "103:",
            "mov    w2, #3",
            "b      200f",

            "104:",
            "mov    w2, #4",
            "b      200f",

            "105:",
            "mov    w2, #5",
            "b      200f",

            "106:",
            "mov    w2, #6",
            "b      200f",

            "107:",
            "mov    w2, #7",
            "b      200f",

            "108:",
            "mov    w2, #8",
            "b      200f",

            "109:",
            "mov    w2, #9",

            "200:",

            in("x1") ptr,
            lateout("x0") result,
            lateout("w2") consumed,
            out("w3") _,
            out("x4") _,
            options(readonly, nostack),
        );
    }
    (result, consumed)
}

/// Decode a u64 from ULEB128 (fallback).
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn decode_uleb128_u64(buf: &[u8]) -> (u64, usize) {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut i = 0;
    loop {
        let byte = buf[i];
        result |= ((byte & 0x7f) as u64) << shift;
        i += 1;
        if byte & 0x80 == 0 {
            return (result, i);
        }
        shift += 7;
    }
}

// u128 support (19 bytes max)

pub const ULEB128_U128_BUF_SIZE: usize = 19;

/// Encode a u128 as ULEB128.
///
/// Returns the number of bytes written.
#[inline(always)]
pub fn encode_uleb128_u128(mut value: u128, buf: &mut [u8; ULEB128_U128_BUF_SIZE]) -> usize {
    let mut i = 0;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            buf[i] = byte;
            return i + 1;
        }
        buf[i] = byte | 0x80;
        i += 1;
    }
}

/// Decode a u128 from ULEB128.
///
/// Returns (value, bytes_consumed).
#[inline(always)]
pub fn decode_uleb128_u128(buf: &[u8]) -> (u128, usize) {
    let mut result: u128 = 0;
    let mut shift = 0;
    let mut i = 0;
    loop {
        let byte = buf[i];
        result |= ((byte & 0x7f) as u128) << shift;
        i += 1;
        if byte & 0x80 == 0 {
            return (result, i);
        }
        shift += 7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uleb128_u32_roundtrip() {
        for value in [0u32, 1, 127, 128, 16383, 16384, 2097151, 2097152, u32::MAX] {
            let mut buf = [0u8; ULEB128_U32_BUF_SIZE];
            let len = encode_uleb128_u32(value, &mut buf);
            let (decoded, consumed) = decode_uleb128_u32(&buf);
            assert_eq!(decoded, value, "value={} len={}", value, len);
            assert_eq!(consumed, len);
        }
    }

    #[test]
    fn test_uleb128_u64_roundtrip() {
        for value in [
            0u64,
            1,
            127,
            128,
            16383,
            16384,
            2097151,
            2097152,
            268435455,
            268435456,
            u32::MAX as u64,
            u64::MAX,
        ] {
            let mut buf = [0u8; ULEB128_U64_BUF_SIZE];
            let len = encode_uleb128_u64(value, &mut buf);
            let (decoded, consumed) = decode_uleb128_u64(&buf);
            assert_eq!(decoded, value, "value={} len={}", value, len);
            assert_eq!(consumed, len);
        }
    }

    #[test]
    fn test_uleb128_u128_roundtrip() {
        for value in [
            0u128,
            1,
            127,
            128,
            u64::MAX as u128,
            u64::MAX as u128 + 1,
            u128::MAX,
        ] {
            let mut buf = [0u8; ULEB128_U128_BUF_SIZE];
            let len = encode_uleb128_u128(value, &mut buf);
            let (decoded, consumed) = decode_uleb128_u128(&buf);
            assert_eq!(decoded, value, "value={} len={}", value, len);
            assert_eq!(consumed, len);
        }
    }
}
