//! Comparison benchmarks: fastvint vs LEB128

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

const U32_TESTS: &[(&str, u32)] = &[("small", 42), ("medium", 10_000), ("large", u32::MAX - 3)];

const I32_TESTS: &[(&str, i32)] = &[
    ("small_pos", 42),
    ("small_neg", -42),
    ("large_pos", i32::MAX - 3),
    ("large_neg", i32::MIN + 3),
];

const U64_TESTS: &[(&str, u64)] = &[
    ("small", 42),
    ("medium", 10_000),
    ("5_byte", 1_000_000_000),
    ("8_byte", 1_000_000_000_000_000),
    ("large", u64::MAX),
];

const U128_TESTS: &[(&str, u128)] = &[
    ("small", 42),
    ("medium", 10_000),
    ("u64_max", u64::MAX as u128),
    ("large", u128::MAX),
];

const I64_TESTS: &[(&str, i64)] = &[
    ("small_pos", 42),
    ("small_neg", -42),
    ("medium_pos", 10_000),
    ("medium_neg", -10_000),
    ("large_pos", i64::MAX),
    ("large_neg", i64::MIN),
];

const I128_TESTS: &[(&str, i128)] = &[
    ("small_pos", 42),
    ("small_neg", -42),
    ("i64_max", i64::MAX as i128),
    ("i64_min", i64::MIN as i128),
    ("large_pos", i128::MAX),
    ("large_neg", i128::MIN),
];

fn bench_encode_u32_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_u32_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U32_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vu32(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("uleb128", name), &value, |b, &v| {
            let mut buf = [0u8; ULEB128_U32_BUF_SIZE];
            b.iter(|| encode_uleb128_u32(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_decode_u32_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_u32_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U32_TESTS {
        let encoded_vlq = encode_vu32(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vu32_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ULEB128_U32_BUF_SIZE];
        encode_uleb128_u32(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("uleb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_uleb128_u32(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_i32_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_i32_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I32_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vi32(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("ileb128", name), &value, |b, &v| {
            let mut buf = [0u8; ILEB128_I32_BUF_SIZE];
            b.iter(|| encode_ileb128_i32(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_decode_i32_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_i32_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I32_TESTS {
        let encoded_vlq = encode_vi32(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vi32_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ILEB128_I32_BUF_SIZE];
        encode_ileb128_i32(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("ileb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_ileb128_i32(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_u64_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_u64_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U64_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vu64(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("uleb128", name), &value, |b, &v| {
            let mut buf = [0u8; ULEB128_U64_BUF_SIZE];
            b.iter(|| encode_uleb128_u64(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_decode_u64_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_u64_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U64_TESTS {
        let encoded_vlq = encode_vu64(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vu64_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ULEB128_U64_BUF_SIZE];
        encode_uleb128_u64(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("uleb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_uleb128_u64(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_i64_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_i64_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I64_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vi64(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("ileb128", name), &value, |b, &v| {
            let mut buf = [0u8; ILEB128_I64_BUF_SIZE];
            b.iter(|| encode_ileb128_i64(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_decode_i64_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_i64_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I64_TESTS {
        let encoded_vlq = encode_vi64(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vi64_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ILEB128_I64_BUF_SIZE];
        encode_ileb128_i64(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("ileb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_ileb128_i64(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_u128_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_u128_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U128_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vu128(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("uleb128", name), &value, |b, &v| {
            let mut buf = [0u8; ULEB128_U128_BUF_SIZE];
            b.iter(|| encode_uleb128_u128(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_decode_u128_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_u128_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in U128_TESTS {
        let encoded_vlq = encode_vu128(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vu128_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ULEB128_U128_BUF_SIZE];
        encode_uleb128_u128(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("uleb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_uleb128_u128(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_i128_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_i128_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I128_TESTS {
        group.bench_with_input(BenchmarkId::new("fastvint", name), &value, |b, &v| {
            b.iter(|| encode_vi128(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("ileb128", name), &value, |b, &v| {
            let mut buf = [0u8; ILEB128_I128_BUF_SIZE];
            b.iter(|| encode_ileb128_i128(black_box(v), &mut buf))
        });
    }
    group.finish();
}

fn bench_decode_i128_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_i128_comparison");
    group.throughput(Throughput::Elements(1));

    for &(name, value) in I128_TESTS {
        let encoded_vlq = encode_vi128(value);
        let vlq_bytes = encoded_vlq.bytes();
        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vi128_slice(black_box(v)))
        });

        let mut leb_buf = [0u8; ILEB128_I128_BUF_SIZE];
        encode_ileb128_i128(value, &mut leb_buf);
        group.bench_with_input(BenchmarkId::new("ileb128", name), &leb_buf, |b, v| {
            b.iter(|| decode_ileb128_i128(black_box(v)))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_encode_u32_comparison,
    bench_decode_u32_comparison,
    bench_encode_i32_comparison,
    bench_decode_i32_comparison,
    bench_encode_u64_comparison,
    bench_decode_u64_comparison,
    bench_encode_i64_comparison,
    bench_decode_i64_comparison,
    bench_encode_u128_comparison,
    bench_decode_u128_comparison,
    bench_encode_i128_comparison,
    bench_decode_i128_comparison,
);
criterion_main!(benches);
