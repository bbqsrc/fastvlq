//! Batch SIMD encoding for large arrays.
//!
//! This module provides optimized batch encoding for processing many values at once,
//! using SIMD instructions where available:
//! - **aarch64**: NEON with vector CLZ (2x u64 per iteration)
//! - **x86_64 AVX-512**: 8x u64 per iteration with `vplzcntq`
//! - **Fallback**: Scalar loop using existing encode functions

#[cfg(all(target_arch = "aarch64", feature = "asm"))]
mod neon;

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx512f",
    target_feature = "avx512cd",
    feature = "asm"
))]
mod avx512;

mod fallback;

#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

/// Maximum encoded size per u64 value (9 bytes).
pub const VU64_MAX_ENCODED_SIZE: usize = 9;

/// Batch encode u64 values to VLQ format.
///
/// Writes encoded bytes to `output` and returns the total bytes written.
/// The output buffer must be at least `values.len() * 9` bytes.
///
/// # Panics
///
/// Panics if `output` is too small to hold the encoded data.
#[inline]
pub fn encode_vu64_batch(values: &[u64], output: &mut [u8]) -> usize {
    assert!(
        output.len() >= values.len() * VU64_MAX_ENCODED_SIZE,
        "output buffer too small: need at least {} bytes, got {}",
        values.len() * VU64_MAX_ENCODED_SIZE,
        output.len()
    );

    #[cfg(all(target_arch = "aarch64", feature = "asm"))]
    {
        return neon::encode_batch(values, output);
    }

    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx512f",
        target_feature = "avx512cd",
        feature = "asm"
    ))]
    {
        return unsafe { avx512::encode_batch(values, output) };
    }

    #[cfg(not(any(
        all(target_arch = "aarch64", feature = "asm"),
        all(
            target_arch = "x86_64",
            target_feature = "avx512f",
            target_feature = "avx512cd",
            feature = "asm"
        )
    )))]
    {
        return fallback::encode_batch(values, output);
    }
}

/// Batch encode u64 values to VLQ format, allocating the output buffer.
///
/// Returns a `Vec<u8>` containing the encoded bytes.
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_vu64_batch_alloc(values: &[u64]) -> Vec<u8> {
    // Allocate worst-case size
    let mut output = vec![0u8; values.len() * VU64_MAX_ENCODED_SIZE];
    let written = encode_vu64_batch(values, &mut output);
    output.truncate(written);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode_vu64;

    #[test]
    fn test_batch_matches_scalar() {
        let values: Vec<u64> = vec![
            0,
            1,
            127,
            128,
            255,
            256,
            10_000,
            100_000,
            1_000_000_000,
            u64::MAX / 2,
            u64::MAX,
        ];

        // Encode with batch
        let batch_output = encode_vu64_batch_alloc(&values);

        // Encode with scalar
        let mut scalar_output = Vec::new();
        for &v in &values {
            let encoded = encode_vu64(v);
            scalar_output.extend_from_slice(&encoded.bytes()[..encoded.len() as usize]);
        }

        assert_eq!(batch_output, scalar_output);
    }

    #[test]
    fn test_batch_empty() {
        let values: Vec<u64> = vec![];
        let output = encode_vu64_batch_alloc(&values);
        assert!(output.is_empty());
    }

    #[test]
    fn test_batch_single() {
        let values = vec![42u64];
        let batch_output = encode_vu64_batch_alloc(&values);

        let encoded = encode_vu64(42);
        let scalar_output = &encoded.bytes()[..encoded.len() as usize];

        assert_eq!(batch_output, scalar_output);
    }

    #[test]
    fn test_batch_all_lengths() {
        // Values that encode to different lengths (1-9 bytes)
        let values: Vec<u64> = vec![
            0,                      // 1 byte
            128,                    // 2 bytes
            16_512,                 // 3 bytes
            2_113_664,              // 4 bytes
            270_549_120,            // 5 bytes
            34_630_287_488,         // 6 bytes
            4_432_676_798_592,      // 7 bytes
            567_382_630_219_904,    // 8 bytes
            72_624_976_668_147_840, // 9 bytes
        ];

        let batch_output = encode_vu64_batch_alloc(&values);

        let mut scalar_output = Vec::new();
        for &v in &values {
            let encoded = encode_vu64(v);
            scalar_output.extend_from_slice(&encoded.bytes()[..encoded.len() as usize]);
        }

        assert_eq!(batch_output, scalar_output);
    }
}
