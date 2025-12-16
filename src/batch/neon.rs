//! aarch64 batch encoding.
//!
//! Processes multiple values per iteration for better pipelining.
//! Uses scalar CLZ (NEON CLZ doesn't support 64-bit elements).

use crate::encode_vu64;

/// Batch encode for aarch64.
///
/// Processes 4 values at a time using scalar CLZ with instruction-level
/// parallelism, as NEON CLZ only supports 8/16/32-bit elements.
#[inline]
pub fn encode_batch(values: &[u64], output: &mut [u8]) -> usize {
    let mut written = 0;
    let chunks = values.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        // Process 4 values - scalar CLZ pipelines well on M-series
        let (v0, v1, v2, v3) = (chunk[0], chunk[1], chunk[2], chunk[3]);

        // Get CLZ for all 4 values (pipelined)
        let clz0 = v0.leading_zeros();
        let clz1 = v1.leading_zeros();
        let clz2 = v2.leading_zeros();
        let clz3 = v3.leading_zeros();

        // Encode all 4 values
        written += encode_with_clz(v0, clz0, &mut output[written..]);
        written += encode_with_clz(v1, clz1, &mut output[written..]);
        written += encode_with_clz(v2, clz2, &mut output[written..]);
        written += encode_with_clz(v3, clz3, &mut output[written..]);
    }

    // Handle remainder with scalar
    for &v in remainder {
        let encoded = encode_vu64(v);
        let len = encoded.len() as usize;
        output[written..written + len].copy_from_slice(&encoded.bytes()[..len]);
        written += len;
    }

    written
}

/// Offsets for each encoded length (1-9 bytes).
const OFFSETS: [u64; 10] = [
    0,                      // unused (index 0)
    0,                      // len=1
    128,                    // len=2
    16_512,                 // len=3
    2_113_664,              // len=4
    270_549_120,            // len=5
    34_630_287_488,         // len=6
    4_432_676_798_592,      // len=7
    567_382_630_219_904,    // len=8
    72_624_976_668_147_840, // len=9
];

/// Prefix byte patterns for each length.
const PREFIX_PATTERNS: [u8; 10] = [
    0x00, // unused
    0x80, // len=1: 1xxx_xxxx
    0x40, // len=2: 01xx_xxxx
    0x20, // len=3: 001x_xxxx
    0x10, // len=4: 0001_xxxx
    0x08, // len=5: 0000_1xxx
    0x04, // len=6: 0000_01xx
    0x02, // len=7: 0000_001x
    0x01, // len=8: 0000_0001
    0x00, // len=9: 0000_0000
];

/// Encode a value given its pre-computed CLZ.
#[inline(always)]
fn encode_with_clz(value: u64, clz: u32, output: &mut [u8]) -> usize {
    // Determine length from CLZ using thresholds
    let len = clz_to_len(clz, value);

    // Get offset for this length
    let offset = OFFSETS[len as usize];
    let adjusted = value.wrapping_sub(offset);

    // Compute prefix byte
    let prefix_pattern = PREFIX_PATTERNS[len as usize];
    let data_bytes = len - 1;

    let prefix = if len == 1 {
        // All 7 bits in prefix
        prefix_pattern | (value as u8)
    } else if len <= 8 {
        // Some bits in prefix, rest in data bytes
        let shift = data_bytes * 8;
        prefix_pattern | ((adjusted >> shift) as u8)
    } else {
        // len=9: just the zero prefix
        0x00
    };

    // Write prefix
    output[0] = prefix;

    // Write data bytes (little-endian)
    if data_bytes > 0 {
        let data_len = data_bytes as usize;
        let bytes = adjusted.to_le_bytes();
        output[1..1 + data_len].copy_from_slice(&bytes[..data_len]);
    }

    len as usize
}

/// Convert CLZ to encoded length.
///
/// CLZ alone isn't sufficient because the thresholds aren't powers of 2.
/// We use CLZ to narrow down, then check against exact thresholds.
#[inline(always)]
fn clz_to_len(clz: u32, value: u64) -> u8 {
    // Quick path using CLZ to estimate
    // CLZ 57-63 -> definitely len=1
    // CLZ 50-56 -> probably len=2, check threshold
    // etc.

    if clz >= 57 {
        return 1;
    }

    // For larger values, check thresholds
    if value < OFFSETS[2] {
        1
    } else if value < OFFSETS[3] {
        2
    } else if value < OFFSETS[4] {
        3
    } else if value < OFFSETS[5] {
        4
    } else if value < OFFSETS[6] {
        5
    } else if value < OFFSETS[7] {
        6
    } else if value < OFFSETS[8] {
        7
    } else if value < OFFSETS[9] {
        8
    } else {
        9
    }
}
