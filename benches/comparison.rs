//! Comparison benchmarks: fastvint vs LEB128 (organized by type)

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fastvint::ileb128::{
    ILEB128_I32_BUF_SIZE, ILEB128_I64_BUF_SIZE, decode_ileb128_i32, decode_ileb128_i64,
    encode_ileb128_i32, encode_ileb128_i64,
};
use fastvint::uleb128::{
    ULEB128_U32_BUF_SIZE, ULEB128_U64_BUF_SIZE, decode_uleb128_u32, decode_uleb128_u64,
    encode_uleb128_u32, encode_uleb128_u64,
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

// u32

fn bench_u32_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("u32_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U32_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vu32(black_box(v)))
        });
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
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vu32_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ULEB128_U32_BUF_SIZE];
        encode_uleb128_u32(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_uleb128_u32(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(BenchmarkId::new("control", name), &control_slice, |b, v| {
            b.iter(|| u32::from_le_bytes(black_box(*v)[..4].try_into().unwrap()))
        });
    }
    group.finish();
}

// i32

fn bench_i32_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("i32_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I32_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vi32(black_box(v)))
        });
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
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vi32_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ILEB128_I32_BUF_SIZE];
        encode_ileb128_i32(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_ileb128_i32(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(BenchmarkId::new("control", name), &control_slice, |b, v| {
            b.iter(|| i32::from_le_bytes(black_box(*v)[..4].try_into().unwrap()))
        });
    }
    group.finish();
}

// u64

fn bench_u64_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("u64_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U64_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vu64(black_box(v)))
        });
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
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vu64_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ULEB128_U64_BUF_SIZE];
        encode_uleb128_u64(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_uleb128_u64(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(BenchmarkId::new("control", name), &control_slice, |b, v| {
            b.iter(|| u64::from_le_bytes(black_box(*v)[..8].try_into().unwrap()))
        });
    }
    group.finish();
}

// i64

fn bench_i64_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("i64_encode");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I64_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vi64(black_box(v)))
        });
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
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vi64_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ILEB128_I64_BUF_SIZE];
        encode_ileb128_i64(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("leb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_ileb128_i64(black_box(v)))
        });

        let control_buf = value.to_le_bytes();
        let control_slice: &[u8] = &control_buf;
        group.bench_with_input(BenchmarkId::new("control", name), &control_slice, |b, v| {
            b.iter(|| i64::from_le_bytes(black_box(*v)[..8].try_into().unwrap()))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_u32_encode,
    bench_u32_decode,
    bench_i32_encode,
    bench_i32_decode,
    bench_u64_encode,
    bench_u64_decode,
    bench_i64_encode,
    bench_i64_decode,
);
criterion_main!(benches);
