//! Unsigned 32-bit VLQ encoding.

use core::fmt::{Debug, Display};

pub(crate) const VU32_BUF_SIZE: usize = 5;

/// Decode length from first byte for u32 (max 5 bytes).
#[inline(always)]
pub(crate) const fn decode_len_vu32(n: u8) -> u8 {
    let len = n.leading_zeros() as u8 + 1;
    if len > 5 { 5 } else { len }
}

/// Encode a u32 in VLQ format using aarch64 inline asm.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn encode_vu32_asm(n: u32) -> (u8, u32) {
    let prefix: u32;
    let data: u32;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Compare against thresholds and branch
            "cmp    w0, #128",
            "b.lo   100f",

            "mov    w4, #0x4080",
            "cmp    w0, w4",
            "b.lo   101f",

            "mov    w4, #0x4080",
            "movk   w4, #0x20, lsl #16",
            "cmp    w0, w4",
            "b.lo   102f",

            "mov    w4, #0x4080",
            "movk   w4, #0x1020, lsl #16",
            "cmp    w0, w4",
            "b.lo   103f",

            // len=5: offset = 270549120 = 0x10204080
            // For u32, high bits are always 0, so prefix is just 0x08
            "mov    w4, #0x4080",
            "movk   w4, #0x1020, lsl #16",
            "sub    w1, w0, w4",           // val = n - offset (this is data)
            "mov    w2, #0x08",            // prefix = 0x08
            "b      200f",

            // len=1: n < 128, offset = 0
            "100:",
            "orr    w2, w0, #0x80",        // prefix = 0x80 | n
            "mov    w1, #0",               // data = 0
            "b      200f",

            // len=2: offset = 128
            "101:",
            "sub    w1, w0, #128",         // val = n - 128
            "lsr    w2, w1, #8",           // high bits
            "orr    w2, w2, #0x40",        // prefix = 0x40 | high
            "and    w1, w1, #0xFF",        // data = low byte
            "b      200f",

            // len=3: offset = 16512 = 0x4080
            "102:",
            "mov    w4, #0x4080",
            "sub    w1, w0, w4",           // val = n - offset
            "lsr    w2, w1, #16",          // high bits
            "orr    w2, w2, #0x20",        // prefix = 0x20 | high
            "and    w1, w1, #0xFFFF",      // data = low 2 bytes
            "b      200f",

            // len=4: offset = 2113664 = 0x204080
            "103:",
            "mov    w4, #0x4080",
            "movk   w4, #0x20, lsl #16",
            "sub    w1, w0, w4",           // val = n - offset
            "lsr    w2, w1, #24",          // high bits
            "orr    w2, w2, #0x10",        // prefix = 0x10 | high
            "ubfx   w1, w1, #0, #24",      // data = low 3 bytes

            "200:",

            inout("w0") n => _,
            out("w1") data,
            out("w2") prefix,
            out("w4") _,
            options(pure, nomem, nostack),
        );
    }
    (prefix as u8, data)
}

/// Encode a u32 in VLQ format.
///
/// Returns Vu32(prefix, packed) where packed contains data bytes in LE order.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn encode_vu32(n: u32) -> Vu32 {
    let (prefix, data) = encode_vu32_asm(n);
    Vu32(prefix, data)
}

/// Encode a u32 in VLQ format using x86_64 inline asm.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn encode_vu32_asm_x86(n: u32) -> (u8, u32) {
    let prefix: u32;
    let data: u32;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Compare against thresholds and branch
            "cmp    {n:e}, 128",
            "jb     100f",

            "cmp    {n:e}, 0x4080",
            "jb     101f",

            "cmp    {n:e}, 0x204080",
            "jb     102f",

            "cmp    {n:e}, 0x10204080",
            "jb     103f",

            // len=5: offset = 270549120 = 0x10204080
            // For u32, high bits are always 0, so prefix is just 0x08
            "mov    {data:e}, {n:e}",
            "sub    {data:e}, 0x10204080",
            "mov    {prefix:e}, 0x08",
            "jmp    200f",

            // len=1: n < 128, offset = 0
            "100:",
            "mov    {prefix:e}, {n:e}",
            "or     {prefix:e}, 0x80",
            "xor    {data:e}, {data:e}",
            "jmp    200f",

            // len=2: offset = 128
            "101:",
            "mov    {data:e}, {n:e}",
            "sub    {data:e}, 128",
            "mov    {prefix:e}, {data:e}",
            "shr    {prefix:e}, 8",
            "or     {prefix:e}, 0x40",
            "and    {data:e}, 0xFF",
            "jmp    200f",

            // len=3: offset = 16512 = 0x4080
            "102:",
            "mov    {data:e}, {n:e}",
            "sub    {data:e}, 0x4080",
            "mov    {prefix:e}, {data:e}",
            "shr    {prefix:e}, 16",
            "or     {prefix:e}, 0x20",
            "and    {data:e}, 0xFFFF",
            "jmp    200f",

            // len=4: offset = 2113664 = 0x204080
            "103:",
            "mov    {data:e}, {n:e}",
            "sub    {data:e}, 0x204080",
            "mov    {prefix:e}, {data:e}",
            "shr    {prefix:e}, 24",
            "or     {prefix:e}, 0x10",
            "and    {data:e}, 0xFFFFFF",

            "200:",

            n = in(reg) n,
            prefix = out(reg) prefix,
            data = out(reg) data,
            options(pure, nomem, nostack),
        );
    }
    (prefix as u8, data)
}

/// Encode a u32 in VLQ format.
///
/// Returns Vu32(prefix, packed) where packed contains data bytes in LE order.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn encode_vu32(n: u32) -> Vu32 {
    let (prefix, data) = encode_vu32_asm_x86(n);
    Vu32(prefix, data)
}

/// Encode a u32 in VLQ format.
///
/// Returns Vu32(prefix, packed) where packed contains data bytes in LE order.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
#[inline(always)]
pub const fn encode_vu32(n: u32) -> Vu32 {
    let n64 = n as u64;

    if n64 < offset!(2) as u64 {
        // len=1: all data in prefix
        Vu32(0x80 | (n as u8), 0)
    } else if n64 < offset!(3) as u64 {
        // len=2: 1 data byte
        let val = n64 - offset!(2) as u64;
        Vu32(0x40 | ((val >> 8) as u8), (val & 0xFF) as u32)
    } else if n64 < offset!(4) as u64 {
        // len=3: 2 data bytes
        let val = n64 - offset!(3) as u64;
        Vu32(0x20 | ((val >> 16) as u8), (val & 0xFFFF) as u32)
    } else if n64 < offset!(5) {
        // len=4: 3 data bytes
        let val = n64 - offset!(4) as u64;
        Vu32(0x10 | ((val >> 24) as u8), (val & 0xFF_FFFF) as u32)
    } else {
        // len=5: 4 data bytes
        let val = n64 - offset!(5);
        Vu32(0x08 | ((val >> 32) as u8), (val & 0xFFFF_FFFF) as u32)
    }
}

// Lookup tables for decode (fallback when no asm available)
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
const OFFSETS: [u64; 6] = [
    0,
    0,
    offset!(2) as u64,
    offset!(3) as u64,
    offset!(4) as u64,
    offset!(5),
];

#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
const MASKS: [u8; 6] = [
    0, 0x7F, // len=1: 7 bits
    0x3F, // len=2: 6 bits
    0x1F, // len=3: 5 bits
    0x0F, // len=4: 4 bits
    0x07, // len=5: 3 bits
];

/// Decode a little-endian VLQ using branchless aarch64 inline asm.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
fn decode_vu32_asm(prefix: u8, data: u32) -> u32 {
    let result: u32;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Get length: clz gives 24-31 for u8 in w reg, so len = clz - 23
            "clz    w4, w3",
            "sub    w4, w4, #23",      // len = 1-5

            // Clamp len to 5 max (for invalid inputs)
            "cmp    w4, #5",
            "csel   w4, w4, w12, le",  // w12 = 5

            // Compute mask = 0xFF >> len
            "mov    w7, #0xFF",
            "lsr    w7, w7, w4",

            // Compute data_bits = (len - 1) * 8
            "sub    w8, w4, #1",
            "lsl    w8, w8, #3",

            // Jump table for offset (16-byte entries)
            "adr    x10, 100f",
            "sub    w11, w4, #1",
            "add    x10, x10, w11, uxtw #4",
            "br     x10",

            // Jump table entries (4 instructions = 16 bytes each)
            "100:",  // len=1: offset = 0
            "mov    w9, #0",
            "b      200f",
            "nop", "nop",

            // len=2: offset = 128
            "mov    w9, #128",
            "b      200f",
            "nop", "nop",

            // len=3: offset = 16512 = 0x4080
            "mov    w9, #0x4080",
            "b      200f",
            "nop", "nop",

            // len=4: offset = 2113664 = 0x204080
            "mov    w9, #0x4080",
            "movk   w9, #0x20, lsl #16",
            "b      200f",
            "nop",

            // len=5: offset = 270549120 = 0x10204080
            "mov    w9, #0x4080",
            "movk   w9, #0x1020, lsl #16",
            "b      200f",
            "nop",

            "200:",
            // result = ((prefix & mask) << data_bits) | data + offset
            "and    w5, w3, w7",
            "lsl    w5, w5, w8",
            "orr    w5, w5, w1",
            "add    w0, w5, w9",

            in("w3") prefix as u32,
            in("w1") data,
            in("w12") 5u32,
            out("w0") result,
            out("w4") _,
            out("w5") _,
            out("w7") _,
            out("w8") _,
            out("w9") _,
            out("x10") _,
            out("w11") _,
            options(pure, nomem, nostack),
        );
    }
    result
}

/// Decode a VLQ back to u32.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_vu32(n: Vu32) -> u32 {
    decode_vu32_asm(n.0, n.1)
}

/// Decode a little-endian VLQ using x86_64 inline asm with LZCNT.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
fn decode_vu32_asm_x86(prefix: u8, data: u32) -> u32 {
    let result: u32;
    // SAFETY: Pure computation, no memory access.
    unsafe {
        core::arch::asm!(
            // Get length: lzcnt on byte gives leading zeros, len = lzcnt - 23
            // But we have prefix in low byte of 32-bit reg, so lzcnt gives 24 + leading zeros of byte
            "movzx  {prefix:e}, {prefix:l}",    // zero-extend byte to 32-bit
            "lzcnt  {len:e}, {prefix:e}",       // lzcnt on 32-bit value
            "sub    {len:e}, 23",               // len = lzcnt - 23

            // Clamp len to 5 max
            "cmp    {len:e}, 5",
            "mov    {tmp:e}, 5",
            "cmova  {len:e}, {tmp:e}",

            // Compute mask = 0xFF >> len
            "mov    {mask:e}, 0xFF",
            "mov    ecx, {len:e}",
            "shr    {mask:e}, cl",

            // Compute data_bits = (len - 1) * 8
            "mov    {shift:e}, {len:e}",
            "sub    {shift:e}, 1",
            "shl    {shift:e}, 3",

            // Jump table: use lea + computed jump
            "lea    {jump:r}, [rip + 100f]",
            "mov    {idx:e}, {len:e}",
            "sub    {idx:e}, 1",
            "imul   {idx:e}, {idx:e}, 16",      // each entry is 16 bytes
            "add    {jump:r}, {idx:r}",
            "jmp    {jump:r}",

            // Jump table entries (must be 16 bytes each)
            // len=1: offset = 0
            "100:",
            "xor    {off:e}, {off:e}",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",  // 7 byte pad

            // len=2: offset = 128
            "mov    {off:e}, 128",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90, 0x90",  // 6 byte pad

            // len=3: offset = 16512 = 0x4080
            "mov    {off:e}, 0x4080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90, 0x90",  // 5 byte pad

            // len=4: offset = 2113664 = 0x204080
            "mov    {off:e}, 0x204080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90, 0x90",  // 4 byte pad

            // len=5: offset = 270549120 = 0x10204080
            "mov    {off:e}, 0x10204080",
            "jmp    200f",
            ".byte 0x90, 0x90, 0x90",  // 3 byte pad

            "200:",
            // result = ((prefix & mask) << data_bits) | data + offset
            "and    {prefix:e}, {mask:e}",
            "mov    ecx, {shift:e}",
            "shl    {prefix:e}, cl",
            "or     {prefix:e}, {data:e}",
            "add    {prefix:e}, {off:e}",

            prefix = inout(reg) prefix as u32 => result,
            data = in(reg) data,
            len = out(reg) _,
            tmp = out(reg) _,
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

/// Decode a VLQ back to u32.
#[cfg(all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm"))]
#[inline(always)]
pub fn decode_vu32(n: Vu32) -> u32 {
    decode_vu32_asm_x86(n.0, n.1)
}

/// Decode a VLQ back to u32.
#[cfg(not(any(
    all(target_arch = "aarch64", feature = "asm"),
    all(target_arch = "x86_64", target_feature = "lzcnt", feature = "asm")
)))]
#[inline(always)]
pub const fn decode_vu32(n: Vu32) -> u32 {
    let len = n.0.leading_zeros() as usize + 1;
    let len = if len > 5 { 5 } else { len };
    let prefix = n.0;
    let data = n.1 as u64;

    if len == 1 {
        (prefix & 0x7F) as u32
    } else {
        let prefix_bits = (prefix & MASKS[len]) as u64;
        let data_bits = (len - 1) * 8;
        (((prefix_bits << data_bits) | data) + OFFSETS[len]) as u32
    }
}

/// Decode a u32 from a byte slice.
///
/// Returns `Some((value, bytes_consumed))` on success, or `None` if the slice is too short.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_vu32_slice(data: &[u8]) -> Option<(u32, usize)> {
    let first = *data.first()?;

    // Fast path: 1-byte encoding (high bit set)
    if first & 0x80 != 0 {
        return Some(((first & 0x7F) as u32, 1));
    }

    // Fast path: 2-byte encoding (bit 6 set)
    if first & 0x40 != 0 {
        let second = *data.get(1)?;
        let val = (((first & 0x3F) as u32) << 8) | (second as u32);
        return Some((val + 128, 2));
    }

    // Fast path: 3-byte encoding (bit 5 set)
    if first & 0x20 != 0 {
        if data.len() < 3 {
            return None;
        }
        let low = u16::from_le_bytes([data[1], data[2]]) as u32;
        let val = (((first & 0x1F) as u32) << 16) | low;
        return Some((val + 16512, 3));
    }

    let len = decode_len_vu32(first) as usize;
    if data.len() < len {
        return None;
    }

    let ptr = data.as_ptr();
    let result: u64;
    unsafe {
        core::arch::asm!(
            // Jump table for load + decode (16-byte entries)
            "adr    x10, 100f",
            "sub    x11, x5, #1",
            "add    x10, x10, x11, lsl #4",
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
            "orr    x1, x1, x6, lsl #16",
            "b      203f",

            // len=5: load 4 bytes, offset=270549120
            // Note: prefix bits are always 0 for valid u32 values
            "ldr    w0, [x4, #1]",
            "b      204f",
            "nop", "nop",

            // Finish paths with offset addition
            "200:",  // len=1 done (offset=0)
            "b      300f",

            "201:",  // len=2: add 128
            "add    x0, x0, #128",
            "b      300f",

            "202:",  // len=3: add 16512
            "mov    x9, #0x4080",
            "add    x0, x0, x9",
            "b      300f",

            "203:",  // len=4: finish load + add 2113664
            "and    w6, w3, #0x0F",
            "orr    x0, x1, x6, lsl #24",
            "mov    x9, #0x4080",
            "movk   x9, #0x20, lsl #16",
            "add    x0, x0, x9",
            "b      300f",

            "204:",  // len=5: add 270549120
            "mov    x9, #0x4080",
            "movk   x9, #0x1020, lsl #16",
            "add    x0, x0, x9",

            "300:",

            in("x3") first as u64,
            in("x4") ptr,
            in("x5") len as u64,
            out("x0") result,
            out("x1") _,
            out("x6") _,
            out("x9") _,
            out("x10") _,
            out("x11") _,
            options(readonly, nostack),
        );
    }
    Some((result as u32, len))
}

/// Decode a u32 from a byte slice.
///
/// Returns `Some((value, bytes_consumed))` on success, or `None` if the slice is too short.
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
#[inline(always)]
pub fn decode_vu32_slice(data: &[u8]) -> Option<(u32, usize)> {
    let first = *data.first()?;

    // Fast path: 1-byte encoding (high bit set)
    if first & 0x80 != 0 {
        return Some(((first & 0x7F) as u32, 1));
    }

    // Fast path: 2-byte encoding (bit 6 set)
    if first & 0x40 != 0 {
        let second = *data.get(1)?;
        let val = (((first & 0x3F) as u32) << 8) | (second as u32);
        return Some((val + 128, 2));
    }

    // Fast path: 3-byte encoding (bit 5 set)
    if first & 0x20 != 0 {
        if data.len() < 3 {
            return None;
        }
        let low = u16::from_le_bytes([data[1], data[2]]) as u32;
        let val = (((first & 0x1F) as u32) << 16) | low;
        return Some((val + 16512, 3));
    }

    let len = decode_len_vu32(first) as usize;
    if data.len() < len {
        return None;
    }

    let mut buf = [0u8; 4];
    buf[..(len - 1)].copy_from_slice(&data[1..len]);
    let packed = u32::from_le_bytes(buf);
    Some((decode_vu32(Vu32(first, packed)), len))
}

/// An unsigned 32-bit integer in variable-length quantity encoding.
///
/// Stored as (prefix_byte, packed_data) to fit in two registers.
#[derive(Clone, Copy)]
pub struct Vu32(pub(crate) u8, pub(crate) u32);

#[allow(clippy::len_without_is_empty)]
impl Vu32 {
    /// Construct a new VLQ instance from the given `u32`.
    #[inline(always)]
    pub fn new(value: u32) -> Vu32 {
        encode_vu32(value)
    }

    /// Retrieve the stored number as `u32`.
    #[inline(always)]
    pub fn get(&self) -> u32 {
        decode_vu32(*self)
    }

    /// Length of the internal representation in bytes.
    #[inline(always)]
    pub const fn len(&self) -> u8 {
        decode_len_vu32(self.0)
    }

    /// Get the raw byte representation of the VLQ instance.
    #[inline(always)]
    pub const fn bytes(&self) -> [u8; VU32_BUF_SIZE] {
        let mut out = [0u8; VU32_BUF_SIZE];
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

impl From<u32> for Vu32 {
    fn from(n: u32) -> Self {
        encode_vu32(n)
    }
}

impl From<Vu32> for u32 {
    fn from(n: Vu32) -> Self {
        decode_vu32(n)
    }
}

impl Display for Vu32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Display::fmt(&u32::from(*self), f)
    }
}

impl Debug for Vu32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let len = self.len() as usize - 1;
        let bytes = self.bytes();
        write!(f, "Vu32(0b")?;
        for x in bytes.iter().take(len) {
            f.write_fmt(core::format_args!("{:08b}_", x))?;
        }
        f.write_fmt(core::format_args!("{:08b})", bytes[len]))
    }
}
