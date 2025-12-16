//! AVX-512 batch encoding for x86_64.
//!
//! Uses AVX-512CD `vplzcntq` to process 8x u64 values per iteration.

use crate::encode_vu64;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Batch encode using AVX-512 (x86_64).
///
/// Processes 8 values at a time using AVX-512 vector LZCNT,
/// then uses scalar logic for the actual encoding.
#[target_feature(enable = "avx512f,avx512cd")]
#[inline]
pub unsafe fn encode_batch(values: &[u64], output: &mut [u8]) -> usize {
    let mut written = 0;
    let chunks = values.chunks_exact(8);
    let remainder = chunks.remainder();

    for chunk in chunks {
        // Load 8x u64 into ZMM register
        let v = _mm512_loadu_si512(chunk.as_ptr() as *const __m512i);

        // Count leading zeros for all 8 values (AVX-512CD)
        let clz = _mm512_lzcnt_epi64(v);

        // Extract CLZ values to array
        let mut clz_arr = [0u64; 8];
        _mm512_storeu_si512(clz_arr.as_mut_ptr() as *mut __m512i, clz);

        // Encode each value using pre-computed CLZ
        for i in 0..8 {
            written += encode_with_clz(chunk[i], clz_arr[i] as u32, &mut output[written..]);
        }
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

/// Data bits in prefix for each length.
const PREFIX_DATA_BITS: [u8; 10] = [
    0, // unused
    7, // len=1: 7 bits in prefix
    6, // len=2: 6 bits in prefix
    5, // len=3: 5 bits in prefix
    4, // len=4: 4 bits in prefix
    3, // len=5: 3 bits in prefix
    2, // len=6: 2 bits in prefix
    1, // len=7: 1 bit in prefix
    0, // len=8: 0 bits in prefix (just marker)
    0, // len=9: no prefix bits
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
#[inline(always)]
fn clz_to_len(clz: u32, value: u64) -> u8 {
    // Quick path using CLZ
    if clz >= 57 {
        return 1;
    }

    // Check thresholds for exact length
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
