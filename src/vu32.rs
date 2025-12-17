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

// Lookup tables for branchless decode (must be static for asm sym operand)
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
static PREFIX_MASKS_32: [u8; 6] = [0, 0x7F, 0x3F, 0x1F, 0x0F, 0x07];
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
const PREFIX_MASKS_32: [u8; 6] = [0, 0x7F, 0x3F, 0x1F, 0x0F, 0x07];

#[cfg(all(target_arch = "aarch64", feature = "asm"))]
static OFFSETS_32: [u32; 6] = [0, 0, 128, 16512, 2113664, 270549120];
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
const OFFSETS_32: [u32; 6] = [0, 0, 128, 16512, 2113664, 270549120];

// Byte masks for branchless decode: masks off unused bytes in 4-byte load
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
static BYTE_MASKS_32: [u32; 6] = [
    0, 0,       // len=1: 0 data bytes (not used in fast path)
    0xFF,       // len=2: 1 data byte
    0xFFFF,     // len=3: 2 data bytes
    0xFFFFFF,   // len=4: 3 data bytes
    0xFFFFFFFF, // len=5: 4 data bytes
];

/// Decode a u32 from a byte slice using unrolled aarch64 assembly.
/// Uses tbnz-based dispatch (test bit and branch) for all lengths.
/// Zero table lookups - offset is baked into per-byte constants.
#[cfg(all(target_arch = "aarch64", feature = "asm"))]
#[inline(always)]
pub fn decode_vu32_slice(data: &[u8]) -> Option<(u32, usize)> {
    if data.is_empty() {
        return None;
    }

    let value: u32;
    let len: usize;

    // SAFETY: We've verified data is not empty. The asm checks bounds via prefix bits.
    // For invalid prefixes (len > data.len()), behavior is undefined but we trust valid input.
    unsafe {
        core::arch::asm!(
            // Load first byte
            "ldrb   w3, [{ptr}]",

            // Dispatch based on prefix bits using tbnz (test bit, branch if not zero)
            // len=1: 1xxxxxxx (bit 7 set)
            // len=2: 01xxxxxx (bit 6 set)
            // len=3: 001xxxxx (bit 5 set)
            // len=4: 0001xxxx (bit 4 set)
            // len=5: 00001xxx (bit 3 set)
            "tbnz   w3, #7, 10f",
            "tbnz   w3, #6, 20f",
            "tbnz   w3, #5, 30f",
            "tbnz   w3, #4, 40f",
            "b      50f",

            // len=1: just mask off high bit
            "10:",
            "and    {out:w}, w3, #0x7F",
            "mov    {len:w}, #1",
            "b      100f",

            // len=2: 1 data byte
            "20:",
            "ldrb   w5, [{ptr}, #1]",
            "add    w5, w5, #0x80",
            "and    w6, w3, #0x3F",
            "add    {out:w}, w5, w6, lsl #8",
            "mov    {len:w}, #2",
            "b      100f",

            // len=3: 2 data bytes - single 16-bit load
            "30:",
            "ldrh   w5, [{ptr}, #1]",           // load bytes 1-2 as LE 16-bit
            "mov    w6, #0x4080",               // combined offset (0x80 + 0x40<<8)
            "add    w5, w5, w6",
            "and    w6, w3, #0x1F",
            "add    {out:w}, w5, w6, lsl #16",
            "mov    {len:w}, #3",
            "b      100f",

            // len=4: 3 data bytes - 32-bit load + mask
            "40:",
            "ldr    w5, [{ptr}, #1]",           // load 4 bytes
            "and    w5, w5, #0xFFFFFF",         // mask to 24 bits
            "mov    w6, #0x4080",
            "movk   w6, #0x20, lsl #16",        // w6 = 0x204080
            "add    w5, w5, w6",
            "and    w6, w3, #0x0F",
            "add    {out:w}, w5, w6, lsl #24",
            "mov    {len:w}, #4",
            "b      100f",

            // len=5: 4 data bytes - single 32-bit load
            "50:",
            "ldr    w5, [{ptr}, #1]",           // load all 4 data bytes
            "mov    w6, #0x4080",
            "movk   w6, #0x1020, lsl #16",      // w6 = 0x10204080
            "add    {out:w}, w5, w6",
            "mov    {len:w}, #5",

            "100:",

            ptr = in(reg) data.as_ptr(),
            out = out(reg) value,
            len = out(reg) len,
            out("w3") _,
            out("w5") _, out("w6") _,
            options(pure, readonly, nostack),
        );
    }

    // Bounds check after decode
    if len > data.len() {
        return None;
    }

    Some((value, len))
}

/// Decode a u32 from a byte slice (fallback for non-aarch64 or no asm feature).
#[cfg(not(all(target_arch = "aarch64", feature = "asm")))]
#[inline(always)]
pub fn decode_vu32_slice(data: &[u8]) -> Option<(u32, usize)> {
    let first = *data.first()?;
    let len = decode_len_vu32(first) as usize;

    if len > 5 {
        return None;
    }
    if data.len() < len {
        return None;
    }

    // Match on length to create fixed-size arrays for from_le_bytes
    let raw = match len {
        1 => 0u32,
        2 => u32::from_le_bytes([data[1], 0, 0, 0]),
        3 => u32::from_le_bytes([data[1], data[2], 0, 0]),
        4 => u32::from_le_bytes([data[1], data[2], data[3], 0]),
        _ => u32::from_le_bytes([data[1], data[2], data[3], data[4]]),
    };

    // Table lookups for prefix mask and offset
    let shift = (len - 1) << 3;
    let prefix_bits = ((first & PREFIX_MASKS_32[len]) as u32) << shift;
    Some(((prefix_bits | raw) + OFFSETS_32[len], len))
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
