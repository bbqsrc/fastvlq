use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fastvint::ileb128::{
    ILEB128_I64_BUF_SIZE, ILEB128_I128_BUF_SIZE, decode_ileb128_i64, decode_ileb128_i128,
    encode_ileb128_i64, encode_ileb128_i128,
};
use fastvint::uleb128::{
    ULEB128_U64_BUF_SIZE, ULEB128_U128_BUF_SIZE, decode_uleb128_u64, decode_uleb128_u128,
    encode_uleb128_u64, encode_uleb128_u128,
};
use fastvint::*;
use std::hint::black_box;

// === decode_slice benchmarks ===

fn bench_decode_vu32_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu32_slice");

    for (name, value) in [("small", 42u32), ("medium", 10_000u32), ("large", u32::MAX)] {
        let encoded = encode_vu32(value);
        let bytes = encoded.bytes();

        group.bench_with_input(BenchmarkId::new("decode", name), &bytes, |b, v| {
            b.iter(|| decode_vu32_slice(black_box(v)))
        });
    }
    group.finish();
}

fn bench_decode_vu64_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu64_slice");

    for (name, value) in [
        ("small", 42u64),                     // 1 byte
        ("medium", 10_000u64),                // 2 bytes
        ("5_byte", 1_000_000_000u64),         // 5 bytes
        ("8_byte", 1_000_000_000_000_000u64), // 8 bytes
        ("large", u64::MAX),                  // 9 bytes
    ] {
        let encoded = encode_vu64(value);
        let bytes = encoded.bytes();

        group.bench_with_input(BenchmarkId::new("decode", name), &bytes, |b, v| {
            b.iter(|| decode_vu64_slice(black_box(v)))
        });
    }
    group.finish();
}

// === decode benchmarks ===

fn bench_decode_vu32(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu32");

    for (name, value) in [("small", 42u32), ("medium", 10_000u32), ("large", u32::MAX)] {
        let encoded = encode_vu32(value);

        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vu32(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vu64(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu64");

    for (name, value) in [
        ("small", 42u64),                     // 1 byte
        ("medium", 10_000u64),                // 2 bytes
        ("5_byte", 1_000_000_000u64),         // 5 bytes
        ("8_byte", 1_000_000_000_000_000u64), // 8 bytes
        ("large", u64::MAX),                  // 9 bytes
    ] {
        let encoded = encode_vu64(value);

        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vu64(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vu128(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu128");

    for (name, value) in [
        ("small", 42u128),
        ("medium", 10_000u128),
        ("large", u128::MAX),
    ] {
        let encoded = encode_vu128(value);

        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vu128(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vi32(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vi32");

    for (name, value) in [
        ("small_pos", 42i32),
        ("small_neg", -42i32),
        ("large_pos", i32::MAX),
        ("large_neg", i32::MIN),
    ] {
        let encoded = encode_vi32(value);

        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vi32(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vi64(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vi64");

    for (name, value) in [
        ("small_pos", 42i64),
        ("small_neg", -42i64),
        ("large_pos", i64::MAX),
        ("large_neg", i64::MIN),
    ] {
        let encoded = encode_vi64(value);

        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vi64(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vi128(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vi128");

    for (name, value) in [
        ("small_pos", 42i128),
        ("small_neg", -42i128),
        ("large_pos", i128::MAX),
        ("large_neg", i128::MIN),
    ] {
        let encoded = encode_vi128(value);

        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vi128(black_box(*v)))
        });
    }
    group.finish();
}

// === encode benchmarks ===

fn bench_encode_vu32(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vu32");

    for (name, value) in [("small", 42u32), ("medium", 10_000u32), ("large", u32::MAX)] {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vu32(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_vu64(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vu64");

    for (name, value) in [
        ("small", 42u64),                     // 1 byte
        ("medium", 10_000u64),                // 2 bytes
        ("5_byte", 1_000_000_000u64),         // 5 bytes
        ("8_byte", 1_000_000_000_000_000u64), // 8 bytes
        ("large", u64::MAX),                  // 9 bytes
    ] {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vu64(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_vu128(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vu128");

    for (name, value) in [
        ("small", 42u128),
        ("medium", 10_000u128),
        ("large", u128::MAX),
    ] {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vu128(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_vi32(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vi32");

    for (name, value) in [
        ("small_pos", 42i32),
        ("small_neg", -42i32),
        ("large_pos", i32::MAX),
        ("large_neg", i32::MIN),
    ] {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vi32(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_vi64(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vi64");

    for (name, value) in [
        ("small_pos", 42i64),
        ("small_neg", -42i64),
        ("large_pos", i64::MAX),
        ("large_neg", i64::MIN),
    ] {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vi64(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_vi128(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vi128");

    for (name, value) in [
        ("small_pos", 42i128),
        ("small_neg", -42i128),
        ("large_pos", i128::MAX),
        ("large_neg", i128::MIN),
    ] {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vi128(black_box(v)))
        });
    }
    group.finish();
}

// === LEB128 vs fastvint comparison benchmarks ===

fn bench_encode_u64_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_u64_comparison");
    group.throughput(Throughput::Elements(1));

    for (name, value) in [
        ("small", 42u64),
        ("medium", 10_000u64),
        ("5_byte", 1_000_000_000u64),
        ("8_byte", 1_000_000_000_000_000u64),
        ("large", u64::MAX),
    ] {
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

    for (name, value) in [
        ("small", 42u64),
        ("medium", 10_000u64),
        ("5_byte", 1_000_000_000u64),
        ("8_byte", 1_000_000_000_000_000u64),
        ("large", u64::MAX),
    ] {
        // fastvint decode from slice
        let encoded_vlq = encode_vu64(value);
        let vlq_bytes = encoded_vlq.bytes();

        group.bench_with_input(BenchmarkId::new("fastvint", name), &vlq_bytes, |b, v| {
            b.iter(|| decode_vu64_slice(black_box(v)))
        });

        // ULEB128 decode from slice
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

    for (name, value) in [
        ("small_pos", 42i64),
        ("small_neg", -42i64),
        ("medium_pos", 10_000i64),
        ("medium_neg", -10_000i64),
        ("large_pos", i64::MAX),
        ("large_neg", i64::MIN),
    ] {
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

    for (name, value) in [
        ("small_pos", 42i64),
        ("small_neg", -42i64),
        ("medium_pos", 10_000i64),
        ("medium_neg", -10_000i64),
        ("large_pos", i64::MAX),
        ("large_neg", i64::MIN),
    ] {
        // fastvint decode
        let encoded_vlq = encode_vi64(value);

        group.bench_with_input(BenchmarkId::new("fastvint", name), &encoded_vlq, |b, v| {
            b.iter(|| decode_vi64(black_box(*v)))
        });

        // ILEB128 decode from slice
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

    for (name, value) in [
        ("small", 42u128),
        ("medium", 10_000u128),
        ("u64_max", u64::MAX as u128),
        ("large", u128::MAX),
    ] {
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

    for (name, value) in [
        ("small", 42u128),
        ("medium", 10_000u128),
        ("u64_max", u64::MAX as u128),
        ("large", u128::MAX),
    ] {
        // fastvint decode
        let encoded_vlq = encode_vu128(value);

        group.bench_with_input(BenchmarkId::new("fastvint", name), &encoded_vlq, |b, v| {
            b.iter(|| decode_vu128(black_box(*v)))
        });

        // ULEB128 decode from slice
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

    for (name, value) in [
        ("small_pos", 42i128),
        ("small_neg", -42i128),
        ("i64_max", i64::MAX as i128),
        ("i64_min", i64::MIN as i128),
        ("large_pos", i128::MAX),
        ("large_neg", i128::MIN),
    ] {
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

    for (name, value) in [
        ("small_pos", 42i128),
        ("small_neg", -42i128),
        ("i64_max", i64::MAX as i128),
        ("i64_min", i64::MIN as i128),
        ("large_pos", i128::MAX),
        ("large_neg", i128::MIN),
    ] {
        // fastvint decode
        let encoded_vlq = encode_vi128(value);

        group.bench_with_input(BenchmarkId::new("fastvint", name), &encoded_vlq, |b, v| {
            b.iter(|| decode_vi128(black_box(*v)))
        });

        // ILEB128 decode from slice
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
    // decode_slice
    bench_decode_vu32_slice,
    bench_decode_vu64_slice,
    // decode
    bench_decode_vu32,
    bench_decode_vu64,
    bench_decode_vu128,
    bench_decode_vi32,
    bench_decode_vi64,
    bench_decode_vi128,
    // encode
    bench_encode_vu32,
    bench_encode_vu64,
    bench_encode_vu128,
    bench_encode_vi32,
    bench_encode_vi64,
    bench_encode_vi128,
    // comparisons
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
