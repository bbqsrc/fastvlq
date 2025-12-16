//! Signed LEB128 encoding (for benchmarking comparison).

pub const ILEB128_I32_BUF_SIZE: usize = 5;
pub const ILEB128_I64_BUF_SIZE: usize = 10;

/// Encode an i32 as ILEB128 (signed LEB128).
///
/// Returns the number of bytes written.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn encode_ileb128_i32(value: i32, buf: &mut [u8; ILEB128_I32_BUF_SIZE]) -> usize {
    let len: usize;
    let ptr = buf.as_mut_ptr();
    // SAFETY: We write at most 5 bytes, buf is exactly 5 bytes.
    unsafe {
        core::arch::asm!(
            // Signed LEB128 encode loop
            // Termination: (value == 0 && !(byte & 0x40)) || (value == -1 && (byte & 0x40))
            "mov    w2, #0",            // byte index

            "100:",  // loop start
            "and    w3, w0, #0x7F",     // byte = value & 0x7F
            "asr    w0, w0, #7",        // value >>= 7 (arithmetic shift)

            // Check termination: value is 0 or -1, and sign bit matches
            "cmp    w0, #0",
            "b.eq   200f",              // if value == 0, check positive termination
            "cmn    w0, #1",            // compare with -1 (cmn = cmp + 1)
            "b.eq   300f",              // if value == -1, check negative termination

            // More bytes needed
            "orr    w3, w3, #0x80",     // set continuation bit
            "strb   w3, [x1, w2, uxtw]",
            "add    w2, w2, #1",
            "b      100b",

            "200:",  // value == 0, check if byte's sign bit is clear
            "tbnz   w3, #6, 400f",      // if bit 6 set, need another byte
            "b      500f",              // done

            "300:",  // value == -1, check if byte's sign bit is set
            "tbz    w3, #6, 400f",      // if bit 6 clear, need another byte
            "b      500f",              // done

            "400:",  // need one more byte
            "orr    w3, w3, #0x80",     // set continuation bit
            "strb   w3, [x1, w2, uxtw]",
            "add    w2, w2, #1",
            // Final byte: value is now 0 or -1
            "and    w3, w0, #0x7F",     // get final byte

            "500:",  // done
            "strb   w3, [x1, w2, uxtw]",
            "add    w2, w2, #1",        // length = index + 1

            inout("w0") value => _,
            in("x1") ptr,
            lateout("w2") len,
            out("w3") _,
            options(nostack),
        );
    }
    len
}

/// Encode an i32 as ILEB128 (fallback).
#[cfg(not(target_arch = "aarch64"))]
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
#[cfg(target_arch = "aarch64")]
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
#[cfg(not(target_arch = "aarch64"))]
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
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn encode_ileb128_i64(value: i64, buf: &mut [u8; ILEB128_I64_BUF_SIZE]) -> usize {
    let len: usize;
    let ptr = buf.as_mut_ptr();
    // SAFETY: We write at most 10 bytes, buf is exactly 10 bytes.
    unsafe {
        core::arch::asm!(
            // Signed LEB128 encode loop
            "mov    w2, #0",            // byte index

            "100:",  // loop start
            "and    w3, w0, #0x7F",     // byte = value & 0x7F
            "asr    x0, x0, #7",        // value >>= 7 (arithmetic shift)

            // Check termination: value is 0 or -1, and sign bit matches
            "cmp    x0, #0",
            "b.eq   200f",              // if value == 0, check positive termination
            "cmn    x0, #1",            // compare with -1
            "b.eq   300f",              // if value == -1, check negative termination

            // More bytes needed
            "orr    w3, w3, #0x80",     // set continuation bit
            "strb   w3, [x1, w2, uxtw]",
            "add    w2, w2, #1",
            "b      100b",

            "200:",  // value == 0, check if byte's sign bit is clear
            "tbnz   w3, #6, 400f",      // if bit 6 set, need another byte
            "b      500f",              // done

            "300:",  // value == -1, check if byte's sign bit is set
            "tbz    w3, #6, 400f",      // if bit 6 clear, need another byte
            "b      500f",              // done

            "400:",  // need one more byte
            "orr    w3, w3, #0x80",     // set continuation bit
            "strb   w3, [x1, w2, uxtw]",
            "add    w2, w2, #1",
            // Final byte: value is now 0 or -1
            "and    w3, w0, #0x7F",     // get final byte

            "500:",  // done
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

/// Encode an i64 as ILEB128 (fallback).
#[cfg(not(target_arch = "aarch64"))]
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
#[cfg(target_arch = "aarch64")]
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
#[cfg(not(target_arch = "aarch64"))]
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
