use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use fastvlq::*;

fn bench_encode_vu32(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vu32");

    for (name, value) in [("small", 42u32), ("medium", 10_000u32), ("large", u32::MAX)] {
        group.bench_with_input(BenchmarkId::new("be", name), &value, |b, &v| {
            b.iter(|| encode_vu32_be(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &value, |b, &v| {
            b.iter(|| encode_vu32_le(black_box(v)))
        });
    }
    group.finish();
}

fn bench_decode_vu32(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu32");

    for (name, value) in [("small", 42u32), ("medium", 10_000u32), ("large", u32::MAX)] {
        let encoded_be = encode_vu32_be(value);
        let encoded_le = encode_vu32_le(value);

        group.bench_with_input(BenchmarkId::new("be", name), &encoded_be, |b, v| {
            b.iter(|| decode_vu32_be(black_box(*v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &encoded_le, |b, v| {
            b.iter(|| decode_vu32_le(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_encode_vu64(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vu64");

    for (name, value) in [
        ("small", 42u64),                       // 1 byte
        ("medium", 10_000u64),                  // 2 bytes
        ("5_byte", 1_000_000_000u64),           // 5 bytes
        ("8_byte", 1_000_000_000_000_000u64),   // 8 bytes
        ("large", u64::MAX),                    // 9 bytes
    ] {
        group.bench_with_input(BenchmarkId::new("be", name), &value, |b, &v| {
            b.iter(|| encode_vu64_be(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &value, |b, &v| {
            b.iter(|| encode_vu64_le(black_box(v)))
        });
    }
    group.finish();
}

fn bench_decode_vu64(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu64");

    for (name, value) in [
        ("small", 42u64),                       // 1 byte
        ("medium", 10_000u64),                  // 2 bytes
        ("5_byte", 1_000_000_000u64),           // 5 bytes
        ("8_byte", 1_000_000_000_000_000u64),   // 8 bytes
        ("large", u64::MAX),                    // 9 bytes
    ] {
        let encoded_be = encode_vu64_be(value);
        let encoded_le = encode_vu64_le(value);

        group.bench_with_input(BenchmarkId::new("be", name), &encoded_be, |b, v| {
            b.iter(|| decode_vu64_be(black_box(*v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &encoded_le, |b, v| {
            b.iter(|| decode_vu64_le(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vu64_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu64_slice");

    for (name, value) in [
        ("small", 42u64),                       // 1 byte
        ("medium", 10_000u64),                  // 2 bytes
        ("5_byte", 1_000_000_000u64),           // 5 bytes
        ("8_byte", 1_000_000_000_000_000u64),   // 8 bytes
        ("large", u64::MAX),                    // 9 bytes
    ] {
        let encoded_be = encode_vu64_be(value);
        let encoded_le = encode_vu64_le(value);
        let bytes_be = encoded_be.bytes();
        let bytes_le = encoded_le.bytes();

        group.bench_with_input(BenchmarkId::new("be", name), &bytes_be, |b, v| {
            b.iter(|| decode_vu64_slice_be(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &bytes_le, |b, v| {
            b.iter(|| decode_vu64_slice_le(black_box(v)))
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
        group.bench_with_input(BenchmarkId::new("be", name), &value, |b, &v| {
            b.iter(|| encode_vu128_be(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &value, |b, &v| {
            b.iter(|| encode_vu128_le(black_box(v)))
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
        let encoded_be = encode_vu128_be(value);
        let encoded_le = encode_vu128_le(value);

        group.bench_with_input(BenchmarkId::new("be", name), &encoded_be, |b, v| {
            b.iter(|| decode_vu128_be(black_box(*v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &encoded_le, |b, v| {
            b.iter(|| decode_vu128_le(black_box(*v)))
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
        group.bench_with_input(BenchmarkId::new("be", name), &value, |b, &v| {
            b.iter(|| encode_vi32_be(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &value, |b, &v| {
            b.iter(|| encode_vi32_le(black_box(v)))
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
        let encoded_be = encode_vi32_be(value);
        let encoded_le = encode_vi32_le(value);

        group.bench_with_input(BenchmarkId::new("be", name), &encoded_be, |b, v| {
            b.iter(|| decode_vi32_be(black_box(*v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &encoded_le, |b, v| {
            b.iter(|| decode_vi32_le(black_box(*v)))
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
        group.bench_with_input(BenchmarkId::new("be", name), &value, |b, &v| {
            b.iter(|| encode_vi64_be(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &value, |b, &v| {
            b.iter(|| encode_vi64_le(black_box(v)))
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
        let encoded_be = encode_vi64_be(value);
        let encoded_le = encode_vi64_le(value);

        group.bench_with_input(BenchmarkId::new("be", name), &encoded_be, |b, v| {
            b.iter(|| decode_vi64_be(black_box(*v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &encoded_le, |b, v| {
            b.iter(|| decode_vi64_le(black_box(*v)))
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
        group.bench_with_input(BenchmarkId::new("be", name), &value, |b, &v| {
            b.iter(|| encode_vi128_be(black_box(v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &value, |b, &v| {
            b.iter(|| encode_vi128_le(black_box(v)))
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
        let encoded_be = encode_vi128_be(value);
        let encoded_le = encode_vi128_le(value);

        group.bench_with_input(BenchmarkId::new("be", name), &encoded_be, |b, v| {
            b.iter(|| decode_vi128_be(black_box(*v)))
        });
        group.bench_with_input(BenchmarkId::new("le", name), &encoded_le, |b, v| {
            b.iter(|| decode_vi128_le(black_box(*v)))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    // bench_encode_vu32,
    // bench_decode_vu32,
    bench_decode_vu64,
    bench_decode_vu64_slice,
    bench_encode_vu64,
    // bench_encode_vu128,
    // bench_decode_vu128,
    // bench_encode_vi32,
    // bench_decode_vi32,
    // bench_encode_vi64,
    // bench_decode_vi64,
    // bench_encode_vi128,
    // bench_decode_vi128,
);
criterion_main!(benches);
