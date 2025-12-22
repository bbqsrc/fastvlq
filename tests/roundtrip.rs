use fastvint::*;

// Short buffer tests - verify decoders handle truncated input gracefully
// Prefix patterns: 1xxx_xxxx=1B, 01xx_xxxx=2B, 001x_xxxx=3B, 0001_xxxx=4B, etc.

#[test]
fn decode_vu32_short_buffer() {
    // Empty buffer
    assert_eq!(decode_vu32_slice(&[]), (0, 0));

    // 2-byte prefix (0x40) with only 1 byte
    assert_eq!(decode_vu32_slice(&[0x40]), (0, 0));

    // 3-byte prefix (0x20) with only 1-2 bytes
    assert_eq!(decode_vu32_slice(&[0x20]), (0, 0));
    assert_eq!(decode_vu32_slice(&[0x20, 0x00]), (0, 0));

    // 4-byte prefix (0x10) with only 1-3 bytes
    assert_eq!(decode_vu32_slice(&[0x10]), (0, 0));
    assert_eq!(decode_vu32_slice(&[0x10, 0x00]), (0, 0));
    assert_eq!(decode_vu32_slice(&[0x10, 0x00, 0x00]), (0, 0));

    // 5-byte prefix (0x08) with only 1-4 bytes
    assert_eq!(decode_vu32_slice(&[0x08]), (0, 0));
    assert_eq!(decode_vu32_slice(&[0x08, 0x00, 0x00]), (0, 0));
    assert_eq!(decode_vu32_slice(&[0x08, 0x00, 0x00, 0x00]), (0, 0));
}

#[test]
fn decode_vu64_short_buffer() {
    // Empty buffer
    assert_eq!(decode_vu64_slice(&[]), (0, 0));

    // 2-byte prefix (0x40) with only 1 byte
    assert_eq!(decode_vu64_slice(&[0x40]), (0, 0));

    // 3-byte prefix (0x20) with only 1-2 bytes
    assert_eq!(decode_vu64_slice(&[0x20]), (0, 0));
    assert_eq!(decode_vu64_slice(&[0x20, 0x00]), (0, 0));

    // 4-byte prefix (0x10) with only 1-3 bytes
    assert_eq!(decode_vu64_slice(&[0x10]), (0, 0));
    assert_eq!(decode_vu64_slice(&[0x10, 0x00]), (0, 0));
    assert_eq!(decode_vu64_slice(&[0x10, 0x00, 0x00]), (0, 0));

    // 5-byte prefix (0x08) with only 1-4 bytes
    assert_eq!(decode_vu64_slice(&[0x08]), (0, 0));
    assert_eq!(decode_vu64_slice(&[0x08, 0x00, 0x00, 0x00]), (0, 0));

    // 6-byte prefix (0x04) with only 1-5 bytes
    assert_eq!(decode_vu64_slice(&[0x04]), (0, 0));
    assert_eq!(decode_vu64_slice(&[0x04, 0x00, 0x00, 0x00, 0x00]), (0, 0));

    // 7-byte prefix (0x02) with only 1-6 bytes
    assert_eq!(decode_vu64_slice(&[0x02]), (0, 0));

    // 8-byte prefix (0x01) with only 1-7 bytes
    assert_eq!(decode_vu64_slice(&[0x01]), (0, 0));

    // 9-byte prefix (0x00) with only 1-8 bytes
    assert_eq!(decode_vu64_slice(&[0x00]), (0, 0));
    assert_eq!(decode_vu64_slice(&[0x00, 0x00, 0x00, 0x00]), (0, 0));
    assert_eq!(
        decode_vu64_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (0, 0)
    );
}

#[test]
fn decode_vi32_short_buffer() {
    // Signed uses same underlying encoding, just zigzag decoded
    assert_eq!(decode_vi32_slice(&[]), (0, 0));
    assert_eq!(decode_vi32_slice(&[0x40]), (0, 0));
    assert_eq!(decode_vi32_slice(&[0x20]), (0, 0));
    assert_eq!(decode_vi32_slice(&[0x20, 0x00]), (0, 0));
}

#[test]
fn decode_vi64_short_buffer() {
    assert_eq!(decode_vi64_slice(&[]), (0, 0));
    assert_eq!(decode_vi64_slice(&[0x40]), (0, 0));
    assert_eq!(decode_vi64_slice(&[0x20]), (0, 0));
    assert_eq!(decode_vi64_slice(&[0x00]), (0, 0));
}

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
    assert_eq!(encode_vu64(u64::MIN).get(), u64::MIN);
    assert_eq!(encode_vu64(0x7F).get(), 0x7F);
    assert_eq!(encode_vu64(0x80).get(), 0x80);
    assert_eq!(encode_vu64(u64::MAX).get(), u64::MAX);

    // Boundary values
    assert_eq!(encode_vu64(0x407F).get(), 0x407F, "max for 2");
    assert_eq!(encode_vu64(0x4080).get(), 0x4080, "min for 3");
    assert_eq!(encode_vu64(0x20_407F).get(), 0x20_407F, "max for 3");
    assert_eq!(encode_vu64(0x20_4080).get(), 0x20_4080, "min for 4");
    assert_eq!(encode_vu64(0x1020_407F).get(), 0x1020_407F, "max for 4");
    assert_eq!(encode_vu64(0x1020_4080).get(), 0x1020_4080, "min for 5");
    assert_eq!(encode_vu64(0x8_1020_407F).get(), 0x8_1020_407F, "max for 5");
    assert_eq!(encode_vu64(0x8_1020_4080).get(), 0x8_1020_4080, "min for 6");
    assert_eq!(
        encode_vu64(0x408_1020_407F).get(),
        0x408_1020_407F,
        "max for 6"
    );
    assert_eq!(
        encode_vu64(0x408_1020_4080).get(),
        0x408_1020_4080,
        "min for 7"
    );
    assert_eq!(
        encode_vu64(0x2_0408_1020_407F).get(),
        0x2_0408_1020_407F,
        "max for 7"
    );
    assert_eq!(
        encode_vu64(0x2_0408_1020_4080).get(),
        0x2_0408_1020_4080,
        "min for 8"
    );
    assert_eq!(
        encode_vu64(0x102_0408_1020_407F).get(),
        0x102_0408_1020_407F,
        "max for 8"
    );
    assert_eq!(
        encode_vu64(0x102_0408_1020_4080).get(),
        0x102_0408_1020_4080,
        "min for 9"
    );
}

#[test]
fn vu32_round_trip() {
    assert_eq!(encode_vu32(0).get(), 0);
    assert_eq!(encode_vu32(127).get(), 127);
    assert_eq!(encode_vu32(128).get(), 128);
    assert_eq!(encode_vu32(u32::MAX).get(), u32::MAX);

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
        let (decoded, consumed) = decode_vu64_slice(&bytes);
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
