//! Fallback scalar implementation for batch encoding.

use crate::encode_vu64;

/// Batch encode using scalar loop (fallback for unsupported platforms).
#[allow(dead_code)]
#[inline]
pub fn encode_batch(values: &[u64], output: &mut [u8]) -> usize {
    let mut written = 0;

    for &v in values {
        let encoded = encode_vu64(v);
        let len = encoded.len() as usize;
        let bytes = encoded.bytes();
        output[written..written + len].copy_from_slice(&bytes[..len]);
        written += len;
    }

    written
}
