use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fastvint::*;
use std::hint::black_box;

// === Test value arrays ===

const U32_TESTS: &[(&str, u32)] = &[("small", 42), ("medium", 10_000), ("large", u32::MAX)];

const U64_TESTS: &[(&str, u64)] = &[
    ("small", 42),
    ("medium", 10_000),
    ("5_byte", 1_000_000_000),
    ("8_byte", 1_000_000_000_000_000),
    ("large", u64::MAX),
];

const I32_TESTS: &[(&str, i32)] = &[
    ("small_pos", 42),
    ("small_neg", -42),
    ("large_pos", i32::MAX),
    ("large_neg", i32::MIN),
];

const I64_TESTS: &[(&str, i64)] = &[
    ("small_pos", 42),
    ("small_neg", -42),
    ("medium_pos", 10_000),
    ("medium_neg", -10_000),
    ("large_pos", i64::MAX),
    ("large_neg", i64::MIN),
];

// === decode_slice benchmarks ===

fn bench_decode_vu32_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu32_slice");
    for &(name, value) in U32_TESTS {
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
    for &(name, value) in U64_TESTS {
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
    for &(name, value) in U32_TESTS {
        let encoded = encode_vu32(value);
        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vu32(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vu64(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vu64");
    for &(name, value) in U64_TESTS {
        let encoded = encode_vu64(value);
        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vu64(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vi32(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vi32");
    for &(name, value) in I32_TESTS {
        let encoded = encode_vi32(value);
        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vi32(black_box(*v)))
        });
    }
    group.finish();
}

fn bench_decode_vi64(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_vi64");
    for &(name, value) in I64_TESTS {
        let encoded = encode_vi64(value);
        group.bench_with_input(BenchmarkId::new("decode", name), &encoded, |b, v| {
            b.iter(|| decode_vi64(black_box(*v)))
        });
    }
    group.finish();
}

// === encode benchmarks ===

fn bench_encode_vu32(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vu32");
    for &(name, value) in U32_TESTS {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vu32(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_vu64(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vu64");
    for &(name, value) in U64_TESTS {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vu64(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_vi32(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vi32");
    for &(name, value) in I32_TESTS {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vi32(black_box(v)))
        });
    }
    group.finish();
}

fn bench_encode_vi64(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_vi64");
    for &(name, value) in I64_TESTS {
        group.bench_with_input(BenchmarkId::new("encode", name), &value, |b, &v| {
            b.iter(|| encode_vi64(black_box(v)))
        });
    }
    group.finish();
}

// === Batch encoding benchmarks ===

fn bench_encode_batch_1k(c: &mut Criterion) {
    use rand::Rng;

    let mut rng = rand::rng();
    let values: Vec<u64> = (0..1024).map(|_| rng.random()).collect();
    let mut output = vec![0u8; values.len() * 9];

    let mut group = c.benchmark_group("encode_batch_1k");
    group.throughput(Throughput::Elements(1024));

    group.bench_function("batch", |b| {
        b.iter(|| encode_vu64_batch(black_box(&values), &mut output))
    });

    group.bench_function("scalar_loop", |b| {
        b.iter(|| {
            let mut written = 0;
            for &v in &values {
                let encoded = encode_vu64(v);
                let len = encoded.len() as usize;
                output[written..written + len].copy_from_slice(&encoded.bytes()[..len]);
                written += len;
            }
            written
        })
    });

    group.finish();
}

fn bench_encode_batch_10k(c: &mut Criterion) {
    use rand::Rng;

    let mut rng = rand::rng();
    let values: Vec<u64> = (0..10240).map(|_| rng.random()).collect();
    let mut output = vec![0u8; values.len() * 9];

    let mut group = c.benchmark_group("encode_batch_10k");
    group.throughput(Throughput::Elements(10240));

    group.bench_function("batch", |b| {
        b.iter(|| encode_vu64_batch(black_box(&values), &mut output))
    });

    group.bench_function("scalar_loop", |b| {
        b.iter(|| {
            let mut written = 0;
            for &v in &values {
                let encoded = encode_vu64(v);
                let len = encoded.len() as usize;
                output[written..written + len].copy_from_slice(&encoded.bytes()[..len]);
                written += len;
            }
            written
        })
    });

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
    bench_decode_vi32,
    bench_decode_vi64,
    // encode
    bench_encode_vu32,
    bench_encode_vu64,
    bench_encode_vi32,
    bench_encode_vi64,
    // batch
    bench_encode_batch_1k,
    bench_encode_batch_10k,
);
criterion_main!(benches);
