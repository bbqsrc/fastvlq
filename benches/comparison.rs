//! Comparison benchmarks: fastvint vs LEB128 (organized by type)

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fastvint::ileb128::{
    ILEB128_I32_BUF_SIZE, ILEB128_I64_BUF_SIZE, ILEB128_I128_BUF_SIZE, decode_ileb128_i32,
    decode_ileb128_i64, decode_ileb128_i128, encode_ileb128_i32, encode_ileb128_i64,
    encode_ileb128_i128,
};
use fastvint::uleb128::{
    ULEB128_U32_BUF_SIZE, ULEB128_U64_BUF_SIZE, ULEB128_U128_BUF_SIZE, decode_uleb128_u32,
    decode_uleb128_u64, decode_uleb128_u128, encode_uleb128_u32, encode_uleb128_u64,
    encode_uleb128_u128,
};
use fastvint::*;
use std::hint::black_box;

const U32_TESTS: &[(&str, u32)] = &[
    ("1_byte", 42),
    ("2_byte", 200),
    ("3_byte", 20_000),
    ("4_byte", 3_000_000),
    ("5_byte", u32::MAX - 3),
];

const I32_TESTS: &[(&str, i32)] = &[
    ("1_byte_pos", 42),
    ("1_byte_neg", -42),
    ("2_byte_pos", 100),
    ("2_byte_neg", -100),
    ("3_byte_pos", 10_000),
    ("3_byte_neg", -10_000),
    ("4_byte_pos", 1_500_000),
    ("4_byte_neg", -1_500_000),
    ("5_byte_pos", i32::MAX - 3),
    ("5_byte_neg", i32::MIN + 3),
];

const U64_TESTS: &[(&str, u64)] = &[
    ("1_byte", 42),
    ("2_byte", 200),
    ("3_byte", 20_000),
    ("4_byte", 3_000_000),
    ("5_byte", 1_000_000_000),
    ("6_byte", 100_000_000_000),
    ("7_byte", 10_000_000_000_000),
    ("8_byte", 1_000_000_000_000_000),
    ("9_byte", u64::MAX - 7),
];

const I64_TESTS: &[(&str, i64)] = &[
    ("1_byte_pos", 42),
    ("1_byte_neg", -42),
    ("2_byte_pos", 100),
    ("2_byte_neg", -100),
    ("3_byte_pos", 10_000),
    ("3_byte_neg", -10_000),
    ("4_byte_pos", 1_500_000),
    ("4_byte_neg", -1_500_000),
    ("5_byte_pos", 500_000_000),
    ("5_byte_neg", -500_000_000),
    ("6_byte_pos", 50_000_000_000),
    ("6_byte_neg", -50_000_000_000),
    ("7_byte_pos", 5_000_000_000_000),
    ("7_byte_neg", -5_000_000_000_000),
    ("8_byte_pos", 500_000_000_000_000),
    ("8_byte_neg", -500_000_000_000_000),
    ("9_byte_pos", i64::MAX - 7),
    ("9_byte_neg", i64::MIN + 7),
];

const U128_TESTS: &[(&str, u128)] = &[
    ("1_byte", 42),
    ("2_byte", 200),
    ("3_byte", 20_000),
    ("4_byte", 3_000_000),
    ("5_byte", 1_000_000_000),
    ("6_byte", 100_000_000_000),
    ("7_byte", 10_000_000_000_000),
    ("8_byte", 1_000_000_000_000_000),
    ("9_byte", 100_000_000_000_000_000),
    ("10_byte", 1u128 << 65),
    ("11_byte", 1u128 << 72),
    ("12_byte", 1u128 << 79),
    ("13_byte", 1u128 << 86),
    ("14_byte", 1u128 << 93),
    ("15_byte", 1u128 << 100),
    ("16_byte", 1u128 << 107),
    ("17_byte", u128::MAX - 3),
];

const I128_TESTS: &[(&str, i128)] = &[
    ("1_byte_pos", 42),
    ("1_byte_neg", -42),
    ("2_byte_pos", 100),
    ("2_byte_neg", -100),
    ("3_byte_pos", 10_000),
    ("3_byte_neg", -10_000),
    ("4_byte_pos", 1_500_000),
    ("4_byte_neg", -1_500_000),
    ("5_byte_pos", 500_000_000),
    ("5_byte_neg", -500_000_000),
    ("6_byte_pos", 50_000_000_000),
    ("6_byte_neg", -50_000_000_000),
    ("7_byte_pos", 5_000_000_000_000),
    ("7_byte_neg", -5_000_000_000_000),
    ("8_byte_pos", 500_000_000_000_000),
    ("8_byte_neg", -500_000_000_000_000),
    ("9_byte_pos", 50_000_000_000_000_000),
    ("9_byte_neg", -50_000_000_000_000_000),
    ("10_byte_pos", 1i128 << 64),
    ("10_byte_neg", -(1i128 << 64)),
    ("11_byte_pos", 1i128 << 71),
    ("11_byte_neg", -(1i128 << 71)),
    ("12_byte_pos", 1i128 << 78),
    ("12_byte_neg", -(1i128 << 78)),
    ("13_byte_pos", 1i128 << 85),
    ("13_byte_neg", -(1i128 << 85)),
    ("14_byte_pos", 1i128 << 92),
    ("14_byte_neg", -(1i128 << 92)),
    ("15_byte_pos", 1i128 << 99),
    ("15_byte_neg", -(1i128 << 99)),
    ("16_byte_pos", 1i128 << 106),
    ("16_byte_neg", -(1i128 << 106)),
    ("17_byte_pos", i128::MAX - 3),
    ("17_byte_neg", i128::MIN + 3),
];

// u32

fn bench_u32_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("u32_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U32_TESTS {
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &value,
            |b, &v| b.iter(|| encode_vu32(black_box(v))),
        );
        group.bench_with_input(BenchmarkId::new("leb128", name), &value, |b, &v| {
            let mut buf = [0u8; ULEB128_U32_BUF_SIZE];
            b.iter(|| encode_uleb128_u32(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_u32_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("u32_decode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U32_TESTS {
        let encoded_vlq = encode_vu32(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &vlq_bytes,
            |b, v| b.iter(|| decode_vu32_slice(black_box(v))),
        );

        let mut leb_buf = [0u8; ULEB128_U32_BUF_SIZE];
        encode_uleb128_u32(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_uleb128_u32(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(
            BenchmarkId::new("control", name),
            &control_slice,
            |b, v| b.iter(|| u32::from_le_bytes(black_box(*v)[..4].try_into().unwrap())),
        );
    }
    group.finish();
}

// i32

fn bench_i32_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("i32_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I32_TESTS {
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &value,
            |b, &v| b.iter(|| encode_vi32(black_box(v))),
        );
        group.bench_with_input(BenchmarkId::new("leb128", name), &value, |b, &v| {
            let mut buf = [0u8; ILEB128_I32_BUF_SIZE];
            b.iter(|| encode_ileb128_i32(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_i32_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("i32_decode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I32_TESTS {
        let encoded_vlq = encode_vi32(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &vlq_bytes,
            |b, v| b.iter(|| decode_vi32_slice(black_box(v))),
        );

        let mut leb_buf = [0u8; ILEB128_I32_BUF_SIZE];
        encode_ileb128_i32(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_ileb128_i32(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(
            BenchmarkId::new("control", name),
            &control_slice,
            |b, v| b.iter(|| i32::from_le_bytes(black_box(*v)[..4].try_into().unwrap())),
        );
    }
    group.finish();
}

// u64

fn bench_u64_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("u64_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U64_TESTS {
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &value,
            |b, &v| b.iter(|| encode_vu64(black_box(v))),
        );
        group.bench_with_input(BenchmarkId::new("leb128", name), &value, |b, &v| {
            let mut buf = [0u8; ULEB128_U64_BUF_SIZE];
            b.iter(|| encode_uleb128_u64(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_u64_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("u64_decode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U64_TESTS {
        let encoded_vlq = encode_vu64(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &vlq_bytes,
            |b, v| b.iter(|| decode_vu64_slice(black_box(v))),
        );

        let mut leb_buf = [0u8; ULEB128_U64_BUF_SIZE];
        encode_uleb128_u64(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_uleb128_u64(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(
            BenchmarkId::new("control", name),
            &control_slice,
            |b, v| b.iter(|| u64::from_le_bytes(black_box(*v)[..8].try_into().unwrap())),
        );
    }
    group.finish();
}

// i64

fn bench_i64_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("i64_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I64_TESTS {
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &value,
            |b, &v| b.iter(|| encode_vi64(black_box(v))),
        );
        group.bench_with_input(BenchmarkId::new("leb128", name), &value, |b, &v| {
            let mut buf = [0u8; ILEB128_I64_BUF_SIZE];
            b.iter(|| encode_ileb128_i64(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_i64_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("i64_decode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I64_TESTS {
        let encoded_vlq = encode_vi64(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &vlq_bytes,
            |b, v| b.iter(|| decode_vi64_slice(black_box(v))),
        );

        let mut leb_buf = [0u8; ILEB128_I64_BUF_SIZE];
        encode_ileb128_i64(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_ileb128_i64(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(
            BenchmarkId::new("control", name),
            &control_slice,
            |b, v| b.iter(|| i64::from_le_bytes(black_box(*v)[..8].try_into().unwrap())),
        );
    }
    group.finish();
}

// u128

fn bench_u128_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("u128_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U128_TESTS {
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &value,
            |b, &v| b.iter(|| encode_vu128(black_box(v))),
        );
        group.bench_with_input(BenchmarkId::new("leb128", name), &value, |b, &v| {
            let mut buf = [0u8; ULEB128_U128_BUF_SIZE];
            b.iter(|| encode_uleb128_u128(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_u128_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("u128_decode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U128_TESTS {
        let encoded_vlq = encode_vu128(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &vlq_bytes,
            |b, v| b.iter(|| decode_vu128_slice(black_box(v))),
        );

        let mut leb_buf = [0u8; ULEB128_U128_BUF_SIZE];
        encode_uleb128_u128(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_uleb128_u128(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(
            BenchmarkId::new("control", name),
            &control_slice,
            |b, v| b.iter(|| u128::from_le_bytes(black_box(*v)[..16].try_into().unwrap())),
        );
    }
    group.finish();
}

// i128

fn bench_i128_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("i128_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I128_TESTS {
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &value,
            |b, &v| b.iter(|| encode_vi128(black_box(v))),
        );
        group.bench_with_input(BenchmarkId::new("leb128", name), &value, |b, &v| {
            let mut buf = [0u8; ILEB128_I128_BUF_SIZE];
            b.iter(|| encode_ileb128_i128(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_i128_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("i128_decode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I128_TESTS {
        let encoded_vlq = encode_vi128(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(
            BenchmarkId::new("fastvint", name),
            &vlq_bytes,
            |b, v| b.iter(|| decode_vi128_slice(black_box(v))),
        );

        let mut leb_buf = [0u8; ILEB128_I128_BUF_SIZE];
        encode_ileb128_i128(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_ileb128_i128(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(
            BenchmarkId::new("control", name),
            &control_slice,
            |b, v| b.iter(|| i128::from_le_bytes(black_box(*v)[..16].try_into().unwrap())),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_u32_encode, bench_u32_decode,
    bench_i32_encode, bench_i32_decode,
    bench_u64_encode, bench_u64_decode,
    bench_i64_encode, bench_i64_decode,
    bench_u128_encode, bench_u128_decode,
    bench_i128_encode, bench_i128_decode
);
criterion_main!(benches);
