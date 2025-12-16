use fastvint::*;

#[test]
fn check_decode_len() {
    // Test decode_len for vu64 through encoding and checking lengths
    assert_eq!(encode_vu64(0x7F).len(), 1, "max for 1");
    assert_eq!(encode_vu64(0x80).len(), 2, "min for 2");
    assert_eq!(encode_vu64(0x407F).len(), 2, "max for 2");
    assert_eq!(encode_vu64(0x4080).len(), 3, "min for 3");
    assert_eq!(encode_vu64(0x20_407F).len(), 3, "max for 3");
    assert_eq!(encode_vu64(0x20_4080).len(), 4, "min for 4");
    assert_eq!(encode_vu64(0x1020_407F).len(), 4, "max for 4");
    assert_eq!(encode_vu64(0x1020_4080).len(), 5, "min for 5");
    assert_eq!(encode_vu64(0x8_1020_407F).len(), 5, "max for 5");
    assert_eq!(encode_vu64(0x8_1020_4080).len(), 6, "min for 6");
    assert_eq!(encode_vu64(0x408_1020_407F).len(), 6, "max for 6");
    assert_eq!(encode_vu64(0x408_1020_4080).len(), 7, "min for 7");
    assert_eq!(encode_vu64(0x2_0408_1020_407F).len(), 7, "max for 7");
    assert_eq!(encode_vu64(0x2_0408_1020_4080).len(), 8, "min for 8");
    assert_eq!(encode_vu64(0x102_0408_1020_407F).len(), 8, "max for 8");
    assert_eq!(encode_vu64(0x102_0408_1020_4080).len(), 9, "min for 9");
    assert_eq!(encode_vu64(u64::MAX).len(), 9, "max for 9");
}

#[test]
fn vu64_round_trip() {
    assert_eq!(decode_vu64(encode_vu64(u64::MIN)), u64::MIN);
    assert_eq!(decode_vu64(encode_vu64(0x7F)), 0x7F);
    assert_eq!(decode_vu64(encode_vu64(0x80)), 0x80);
    assert_eq!(decode_vu64(encode_vu64(u64::MAX)), u64::MAX);

    // Boundary values
    assert_eq!(decode_vu64(encode_vu64(0x407F)), 0x407F, "max for 2");
    assert_eq!(decode_vu64(encode_vu64(0x4080)), 0x4080, "min for 3");
    assert_eq!(decode_vu64(encode_vu64(0x20_407F)), 0x20_407F, "max for 3");
    assert_eq!(decode_vu64(encode_vu64(0x20_4080)), 0x20_4080, "min for 4");
    assert_eq!(
        decode_vu64(encode_vu64(0x1020_407F)),
        0x1020_407F,
        "max for 4"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x1020_4080)),
        0x1020_4080,
        "min for 5"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x8_1020_407F)),
        0x8_1020_407F,
        "max for 5"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x8_1020_4080)),
        0x8_1020_4080,
        "min for 6"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x408_1020_407F)),
        0x408_1020_407F,
        "max for 6"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x408_1020_4080)),
        0x408_1020_4080,
        "min for 7"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x2_0408_1020_407F)),
        0x2_0408_1020_407F,
        "max for 7"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x2_0408_1020_4080)),
        0x2_0408_1020_4080,
        "min for 8"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x102_0408_1020_407F)),
        0x102_0408_1020_407F,
        "max for 8"
    );
    assert_eq!(
        decode_vu64(encode_vu64(0x102_0408_1020_4080)),
        0x102_0408_1020_4080,
        "min for 9"
    );
}

#[test]
fn vu32_round_trip() {
    assert_eq!(decode_vu32(encode_vu32(0)), 0);
    assert_eq!(decode_vu32(encode_vu32(127)), 127);
    assert_eq!(decode_vu32(encode_vu32(128)), 128);
    assert_eq!(decode_vu32(encode_vu32(u32::MAX)), u32::MAX);

    // Check lengths
    assert_eq!(encode_vu32(0).len(), 1);
    assert_eq!(encode_vu32(127).len(), 1);
    assert_eq!(encode_vu32(128).len(), 2);
    assert_eq!(encode_vu32(u32::MAX).len(), 5);
}

#[test]
fn vi32_round_trip() {
    assert_eq!(decode_vi32(encode_vi32(0)), 0);
    assert_eq!(decode_vi32(encode_vi32(1)), 1);
    assert_eq!(decode_vi32(encode_vi32(-1)), -1);
    assert_eq!(decode_vi32(encode_vi32(i32::MIN)), i32::MIN);
    assert_eq!(decode_vi32(encode_vi32(i32::MAX)), i32::MAX);

    // Small values should be compact
    assert_eq!(encode_vi32(0).len(), 1);
    assert_eq!(encode_vi32(1).len(), 1);
    assert_eq!(encode_vi32(-1).len(), 1);
}

#[test]
fn vi64_round_trip() {
    assert_eq!(decode_vi64(encode_vi64(0)), 0);
    assert_eq!(decode_vi64(encode_vi64(1)), 1);
    assert_eq!(decode_vi64(encode_vi64(-1)), -1);
    assert_eq!(decode_vi64(encode_vi64(i64::MIN)), i64::MIN);
    assert_eq!(decode_vi64(encode_vi64(i64::MAX)), i64::MAX);

    // Small values should be compact
    assert_eq!(encode_vi64(0).len(), 1);
    assert_eq!(encode_vi64(1).len(), 1);
    assert_eq!(encode_vi64(-1).len(), 1);
}

#[test]
fn vu128_round_trip_small() {
    // Small values (same as u64 range)
    assert_eq!(decode_vu128(encode_vu128(0)), 0);
    assert_eq!(decode_vu128(encode_vu128(127)), 127);
    assert_eq!(decode_vu128(encode_vu128(128)), 128);
    assert_eq!(
        decode_vu128(encode_vu128(u64::MAX as u128)),
        u64::MAX as u128
    );
}

#[test]
fn vu128_round_trip_large() {
    // Large values (beyond u64 range)
    let val = u64::MAX as u128 + 1;
    assert_eq!(decode_vu128(encode_vu128(val)), val);

    assert_eq!(decode_vu128(encode_vu128(u128::MAX)), u128::MAX);
}

#[test]
fn vi128_round_trip() {
    assert_eq!(decode_vi128(encode_vi128(0)), 0);
    assert_eq!(decode_vi128(encode_vi128(1)), 1);
    assert_eq!(decode_vi128(encode_vi128(-1)), -1);
    assert_eq!(decode_vi128(encode_vi128(i128::MIN)), i128::MIN);
    assert_eq!(decode_vi128(encode_vi128(i128::MAX)), i128::MAX);
}

#[test]
fn vu64_bytes_roundtrip() {
    // Test that bytes() produces correct wire format by decoding from slice
    let test_values: &[u64] = &[
        0,
        1,
        127,     // max 1-byte
        128,     // min 2-byte
        500,     // 2-byte with high bits in prefix
        16511,   // max 2-byte
        16512,   // min 3-byte
        100_000, // 3-byte
        u32::MAX as u64,
        u64::MAX / 2,
        u64::MAX,
    ];

    for &val in test_values {
        let encoded = encode_vu64(val);
        let bytes = encoded.bytes();
        let len = encoded.len() as usize;

        // Decode from slice should match original value
        let (decoded, consumed) = decode_vu64_slice(&bytes).unwrap();
        assert_eq!(
            decoded,
            val,
            "bytes() roundtrip failed for {}: got {}, bytes={:02x?}",
            val,
            decoded,
            &bytes[..len]
        );
        assert_eq!(consumed, len, "consumed length mismatch for {}", val);
    }
}

#[test]
fn vu128_bytes_roundtrip() {
    // Test that bytes() produces correct wire format by decoding from slice
    let test_values: &[u128] = &[
        0,
        1,
        127,                  // max 1-byte
        128,                  // min 2-byte
        500,                  // 2-byte
        16511,                // max 2-byte
        16512,                // min 3-byte
        100_000,              // 3-byte
        u32::MAX as u128,     // 5-byte
        u64::MAX as u128,     // 9-byte (max for standard format)
        u64::MAX as u128 + 1, // 10-byte (min for extended format)
        u128::MAX / 2,        // large extended
        u128::MAX,            // max
    ];

    for &val in test_values {
        let encoded = encode_vu128(val);
        let bytes = encoded.bytes();
        let len = encoded.len() as usize;

        // Decode from slice should match original value
        let (decoded, consumed) = decode_vu128_slice(&bytes).unwrap();
        assert_eq!(
            decoded,
            val,
            "bytes() roundtrip failed for {}: got {}, bytes={:02x?}",
            val,
            decoded,
            &bytes[..len]
        );
        assert_eq!(consumed, len, "consumed length mismatch for {}", val);
    }
}
