use fastvlq::*;

#[test]
fn check_decode_len_be() {
    // Test decode_len for vu64 through encoding and checking lengths
    assert_eq!(encode_vu64_be(0x7F).len(), 1, "max for 1");
    assert_eq!(encode_vu64_be(0x80).len(), 2, "min for 2");
    assert_eq!(encode_vu64_be(0x407F).len(), 2, "max for 2");
    assert_eq!(encode_vu64_be(0x4080).len(), 3, "min for 3");
    assert_eq!(encode_vu64_be(0x20_407F).len(), 3, "max for 3");
    assert_eq!(encode_vu64_be(0x20_4080).len(), 4, "min for 4");
    assert_eq!(encode_vu64_be(0x1020_407F).len(), 4, "max for 4");
    assert_eq!(encode_vu64_be(0x1020_4080).len(), 5, "min for 5");
    assert_eq!(encode_vu64_be(0x8_1020_407F).len(), 5, "max for 5");
    assert_eq!(encode_vu64_be(0x8_1020_4080).len(), 6, "min for 6");
    assert_eq!(encode_vu64_be(0x408_1020_407F).len(), 6, "max for 6");
    assert_eq!(encode_vu64_be(0x408_1020_4080).len(), 7, "min for 7");
    assert_eq!(encode_vu64_be(0x2_0408_1020_407F).len(), 7, "max for 7");
    assert_eq!(encode_vu64_be(0x2_0408_1020_4080).len(), 8, "min for 8");
    assert_eq!(encode_vu64_be(0x102_0408_1020_407F).len(), 8, "max for 8");
    assert_eq!(encode_vu64_be(0x102_0408_1020_4080).len(), 9, "min for 9");
    assert_eq!(encode_vu64_be(u64::MAX).len(), 9, "max for 9");
}

#[test]
fn vu64_round_trip_be() {
    assert_eq!(decode_vu64_be(encode_vu64_be(u64::MIN)), u64::MIN);
    // 1-byte: 0x00 to 0x7F
    assert_eq!(encode_vu64_be(0x7F).len(), 1);
    assert_eq!(decode_vu64_be(encode_vu64_be(0x7F)), 0x7F, "max for 1");
    // 2-byte: 0x80 to 0x407F
    assert_eq!(encode_vu64_be(0x80).len(), 2);
    assert_eq!(decode_vu64_be(encode_vu64_be(0x80)), 0x80, "min for 2");
    assert_eq!(encode_vu64_be(0x407F).len(), 2);
    assert_eq!(decode_vu64_be(encode_vu64_be(0x407F)), 0x407F, "max for 2");
    // 3-byte: 0x4080 to 0x20_407F
    assert_eq!(encode_vu64_be(0x4080).len(), 3);
    assert_eq!(decode_vu64_be(encode_vu64_be(0x4080)), 0x4080, "min for 3");
    assert_eq!(encode_vu64_be(0x20_407F).len(), 3);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x20_407F)),
        0x20_407F,
        "max for 3"
    );
    // 4-byte: 0x20_4080 to 0x1020_407F
    assert_eq!(encode_vu64_be(0x20_4080).len(), 4);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x20_4080)),
        0x20_4080,
        "min for 4"
    );
    assert_eq!(encode_vu64_be(0x1020_407F).len(), 4);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x1020_407F)),
        0x1020_407F,
        "max for 4"
    );
    // 5-byte: 0x1020_4080 to 0x8_1020_407F
    assert_eq!(encode_vu64_be(0x1020_4080).len(), 5);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x1020_4080)),
        0x1020_4080,
        "min for 5"
    );
    assert_eq!(encode_vu64_be(0x8_1020_407F).len(), 5);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x8_1020_407F)),
        0x8_1020_407F,
        "max for 5"
    );
    // 6-byte: 0x8_1020_4080 to 0x408_1020_407F
    assert_eq!(encode_vu64_be(0x8_1020_4080).len(), 6);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x8_1020_4080)),
        0x8_1020_4080,
        "min for 6"
    );
    assert_eq!(encode_vu64_be(0x408_1020_407F).len(), 6);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x408_1020_407F)),
        0x408_1020_407F,
        "max for 6"
    );
    // 7-byte: 0x408_1020_4080 to 0x2_0408_1020_407F
    assert_eq!(encode_vu64_be(0x408_1020_4080).len(), 7);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x408_1020_4080)),
        0x408_1020_4080,
        "min for 7"
    );
    assert_eq!(encode_vu64_be(0x2_0408_1020_407F).len(), 7);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x2_0408_1020_407F)),
        0x2_0408_1020_407F,
        "max for 7"
    );
    // 8-byte: 0x2_0408_1020_4080 to 0x102_0408_1020_407F
    assert_eq!(encode_vu64_be(0x2_0408_1020_4080).len(), 8);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x2_0408_1020_4080)),
        0x2_0408_1020_4080,
        "min for 8"
    );
    assert_eq!(encode_vu64_be(0x102_0408_1020_407F).len(), 8);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x102_0408_1020_407F)),
        0x102_0408_1020_407F,
        "max for 8"
    );
    // 9-byte: 0x102_0408_1020_4080 to u64::MAX
    assert_eq!(encode_vu64_be(0x102_0408_1020_4080).len(), 9);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(0x102_0408_1020_4080)),
        0x102_0408_1020_4080,
        "min for 9"
    );
    assert_eq!(encode_vu64_be(u64::MAX).len(), 9);
    assert_eq!(decode_vu64_be(encode_vu64_be(u64::MAX)), u64::MAX);
    assert_eq!(
        decode_vu64_be(encode_vu64_be(i64::MIN as u64)) as i64,
        i64::MIN
    );

    assert_eq!(1, decode_vu64_be(encode_vu64_be(0x1)), "1");
    assert_eq!(0, decode_vu64_be(encode_vu64_be(0x0)), "0");
    assert_eq!(0x011, decode_vu64_be(encode_vu64_be(0x011)), "2");
    assert_eq!(0xFF221122, decode_vu64_be(encode_vu64_be(0xFF221122)), "3");
    assert_eq!(
        0x11FF_FFFF_FFFF_FFFF,
        decode_vu64_be(encode_vu64_be(0x11FF_FFFF_FFFF_FFFF)),
        "4"
    );
    assert_eq!(
        0x1011_1111_1111_1111,
        decode_vu64_be(encode_vu64_be(0x1011_1111_1111_1111)),
        "5"
    );
    assert_eq!(u64::MAX, decode_vu64_be(encode_vu64_be(u64::MAX)), "max");
}

#[test]
fn vu64_round_trip_le() {
    assert_eq!(decode_vu64_le(encode_vu64_le(u64::MIN)), u64::MIN);
    assert_eq!(decode_vu64_le(encode_vu64_le(0x7F)), 0x7F);
    assert_eq!(decode_vu64_le(encode_vu64_le(0x80)), 0x80);
    assert_eq!(decode_vu64_le(encode_vu64_le(u64::MAX)), u64::MAX);
}

#[test]
fn vu32_round_trip_be() {
    assert_eq!(decode_vu32_be(encode_vu32_be(0)), 0);
    assert_eq!(decode_vu32_be(encode_vu32_be(127)), 127);
    assert_eq!(decode_vu32_be(encode_vu32_be(128)), 128);
    assert_eq!(decode_vu32_be(encode_vu32_be(u32::MAX)), u32::MAX);

    // Check lengths
    assert_eq!(encode_vu32_be(0).len(), 1);
    assert_eq!(encode_vu32_be(127).len(), 1);
    assert_eq!(encode_vu32_be(128).len(), 2);
    assert_eq!(encode_vu32_be(u32::MAX).len(), 5);
}

#[test]
fn vu32_round_trip_le() {
    assert_eq!(decode_vu32_le(encode_vu32_le(0)), 0);
    assert_eq!(decode_vu32_le(encode_vu32_le(127)), 127);
    assert_eq!(decode_vu32_le(encode_vu32_le(128)), 128);
    assert_eq!(decode_vu32_le(encode_vu32_le(u32::MAX)), u32::MAX);
}

#[test]
fn vi32_round_trip_be() {
    assert_eq!(decode_vi32_be(encode_vi32_be(0)), 0);
    assert_eq!(decode_vi32_be(encode_vi32_be(1)), 1);
    assert_eq!(decode_vi32_be(encode_vi32_be(-1)), -1);
    assert_eq!(decode_vi32_be(encode_vi32_be(i32::MIN)), i32::MIN);
    assert_eq!(decode_vi32_be(encode_vi32_be(i32::MAX)), i32::MAX);

    // Small values should be compact
    assert_eq!(encode_vi32_be(0).len(), 1);
    assert_eq!(encode_vi32_be(1).len(), 1);
    assert_eq!(encode_vi32_be(-1).len(), 1);
}

#[test]
fn vi32_round_trip_le() {
    assert_eq!(decode_vi32_le(encode_vi32_le(0)), 0);
    assert_eq!(decode_vi32_le(encode_vi32_le(1)), 1);
    assert_eq!(decode_vi32_le(encode_vi32_le(-1)), -1);
    assert_eq!(decode_vi32_le(encode_vi32_le(i32::MIN)), i32::MIN);
    assert_eq!(decode_vi32_le(encode_vi32_le(i32::MAX)), i32::MAX);
}

#[test]
fn vi64_round_trip_be() {
    assert_eq!(decode_vi64_be(encode_vi64_be(0)), 0);
    assert_eq!(decode_vi64_be(encode_vi64_be(1)), 1);
    assert_eq!(decode_vi64_be(encode_vi64_be(-1)), -1);
    assert_eq!(decode_vi64_be(encode_vi64_be(i64::MIN)), i64::MIN);
    assert_eq!(decode_vi64_be(encode_vi64_be(i64::MAX)), i64::MAX);

    // Small values should be compact
    assert_eq!(encode_vi64_be(0).len(), 1);
    assert_eq!(encode_vi64_be(1).len(), 1);
    assert_eq!(encode_vi64_be(-1).len(), 1);
}

#[test]
fn vi64_round_trip_le() {
    assert_eq!(decode_vi64_le(encode_vi64_le(0)), 0);
    assert_eq!(decode_vi64_le(encode_vi64_le(1)), 1);
    assert_eq!(decode_vi64_le(encode_vi64_le(-1)), -1);
    assert_eq!(decode_vi64_le(encode_vi64_le(i64::MIN)), i64::MIN);
    assert_eq!(decode_vi64_le(encode_vi64_le(i64::MAX)), i64::MAX);
}

#[test]
fn vu128_round_trip_small_be() {
    // Small values (same as u64 range)
    assert_eq!(decode_vu128_be(encode_vu128_be(0)), 0);
    assert_eq!(decode_vu128_be(encode_vu128_be(127)), 127);
    assert_eq!(decode_vu128_be(encode_vu128_be(128)), 128);
    assert_eq!(
        decode_vu128_be(encode_vu128_be(u64::MAX as u128)),
        u64::MAX as u128
    );
}

#[test]
fn vu128_round_trip_small_le() {
    assert_eq!(decode_vu128_le(encode_vu128_le(0)), 0);
    assert_eq!(decode_vu128_le(encode_vu128_le(127)), 127);
    assert_eq!(decode_vu128_le(encode_vu128_le(128)), 128);
    assert_eq!(
        decode_vu128_le(encode_vu128_le(u64::MAX as u128)),
        u64::MAX as u128
    );
}

#[test]
fn vu128_round_trip_large_be() {
    // Large values (beyond u64 range)
    let val = u64::MAX as u128 + 1;
    assert_eq!(decode_vu128_be(encode_vu128_be(val)), val);

    assert_eq!(decode_vu128_be(encode_vu128_be(u128::MAX)), u128::MAX);
}

#[test]
fn vu128_round_trip_large_le() {
    let val = u64::MAX as u128 + 1;
    assert_eq!(decode_vu128_le(encode_vu128_le(val)), val);

    assert_eq!(decode_vu128_le(encode_vu128_le(u128::MAX)), u128::MAX);
}

#[test]
fn vi128_round_trip_be() {
    assert_eq!(decode_vi128_be(encode_vi128_be(0)), 0);
    assert_eq!(decode_vi128_be(encode_vi128_be(1)), 1);
    assert_eq!(decode_vi128_be(encode_vi128_be(-1)), -1);
    assert_eq!(decode_vi128_be(encode_vi128_be(i128::MIN)), i128::MIN);
    assert_eq!(decode_vi128_be(encode_vi128_be(i128::MAX)), i128::MAX);
}

#[test]
fn vi128_round_trip_le() {
    assert_eq!(decode_vi128_le(encode_vi128_le(0)), 0);
    assert_eq!(decode_vi128_le(encode_vi128_le(1)), 1);
    assert_eq!(decode_vi128_le(encode_vi128_le(-1)), -1);
    assert_eq!(decode_vi128_le(encode_vi128_le(i128::MIN)), i128::MIN);
    assert_eq!(decode_vi128_le(encode_vi128_le(i128::MAX)), i128::MAX);
}

#[test]
fn vu64_bytes_roundtrip_le() {
    // Test that bytes() produces correct wire format by decoding from slice
    let test_values: &[u64] = &[
        0,
        1,
        127,        // max 1-byte
        128,        // min 2-byte
        500,        // 2-byte with high bits in prefix
        16511,      // max 2-byte
        16512,      // min 3-byte
        100_000,    // 3-byte
        u32::MAX as u64,
        u64::MAX / 2,
        u64::MAX,
    ];

    for &val in test_values {
        let encoded = encode_vu64_le(val);
        let bytes = encoded.bytes();
        let len = encoded.len() as usize;

        // Decode from slice should match original value
        let (decoded, consumed) = decode_vu64_slice_le(&bytes).unwrap();
        assert_eq!(
            decoded, val,
            "bytes() roundtrip failed for {}: got {}, bytes={:02x?}",
            val, decoded, &bytes[..len]
        );
        assert_eq!(consumed, len, "consumed length mismatch for {}", val);
    }
}

#[test]
fn vu64_bytes_roundtrip_be() {
    // Test that bytes() produces correct wire format by decoding from slice
    let test_values: &[u64] = &[
        0,
        1,
        127,
        128,
        500,
        16511,
        16512,
        100_000,
        u32::MAX as u64,
        u64::MAX / 2,
        u64::MAX,
    ];

    for &val in test_values {
        let encoded = encode_vu64_be(val);
        let bytes = encoded.bytes();
        let len = encoded.len() as usize;

        let (decoded, consumed) = decode_vu64_slice_be(&bytes).unwrap();
        assert_eq!(
            decoded, val,
            "bytes() roundtrip failed for {}: got {}, bytes={:02x?}",
            val, decoded, &bytes[..len]
        );
        assert_eq!(consumed, len, "consumed length mismatch for {}", val);
    }
}
