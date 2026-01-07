use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mhub_storage::{Compression, Storage};
use std::hint::black_box;
use std::time::Duration;
use tempfile::TempDir;

// ============================================================================
// Benchmark: Path Resolution & Security Validation
// ============================================================================

fn bench_path_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_resolution");

    let temp = TempDir::new().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        Storage::builder().root(temp.path()).create(true).connect().await.unwrap()
    });

    group.bench_function("simple_path", |b| {
        b.iter(|| {
            black_box(storage.resolve("test.dat").unwrap());
        });
    });

    group.bench_function("nested_path", |b| {
        b.iter(|| {
            black_box(storage.resolve("foo/bar/baz/test.dat").unwrap());
        });
    });

    group.bench_function("sharded_path", |b| {
        b.iter(|| {
            black_box(storage.resolve("abcd1234.dat").unwrap());
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark: Compression Performance
// ============================================================================

fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    // Test different data sizes
    let sizes = [("1KB", 1024), ("10KB", 10 * 1024), ("100KB", 100 * 1024), ("1MB", 1024 * 1024)];

    for (name, size) in sizes {
        let data: Vec<u8> = (0..size).map(|i| u8::try_from(i % 256).unwrap()).collect();

        let throughput = u64::try_from(size).unwrap_or(u64::MAX);
        group.throughput(Throughput::Bytes(throughput));

        group.bench_with_input(BenchmarkId::new("compress", name), &data, |b, data| {
            b.iter(|| {
                black_box(lz4_flex::compress_prepend_size(data));
            });
        });

        let compressed = lz4_flex::compress_prepend_size(&data);
        group.bench_with_input(
            BenchmarkId::new("decompress", name),
            &compressed,
            |b, compressed| {
                b.iter(|| {
                    black_box(lz4_flex::decompress_size_prepended(compressed).unwrap());
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark: File I/O Operations
// ============================================================================

fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");
    group.measurement_time(Duration::from_secs(10));

    let temp = TempDir::new().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Test different file sizes
    let sizes = [("1KB", 1024), ("10KB", 10 * 1024), ("100KB", 100 * 1024)];

    for (name, size) in sizes {
        let data: Vec<u8> = (0..size).map(|i| u8::try_from(i % 256).unwrap()).collect();

        // Benchmark: Write without compression
        group.bench_with_input(BenchmarkId::new("write_uncompressed", name), &data, |b, data| {
            let storage = rt.block_on(async {
                Storage::builder().root(temp.path()).create(true).connect().await.unwrap()
            });

            b.to_async(&rt).iter(|| async {
                storage.write(format!("bench_{name}.dat"), data).await.unwrap();
            });
        });

        // Benchmark: Write with compression
        group.bench_with_input(BenchmarkId::new("write_compressed", name), &data, |b, data| {
            let storage = rt.block_on(async {
                Storage::builder()
                    .root(temp.path())
                    .create(true)
                    .compression(Compression::Lz4)
                    .connect()
                    .await
                    .unwrap()
            });

            b.to_async(&rt).iter(|| async {
                storage.write(format!("bench_{name}_compressed.dat"), data).await.unwrap();
            });
        });

        // Setup: Write a file to read benchmarks
        let storage = rt.block_on(async {
            let s = Storage::builder().root(temp.path()).create(true).connect().await.unwrap();
            s.write(format!("read_bench_{name}.dat"), &data).await.unwrap();
            s
        });

        // Benchmark: Read without compression
        group.bench_function(BenchmarkId::new("read_uncompressed", name), |b| {
            b.to_async(&rt).iter(|| async {
                black_box(storage.read(format!("read_bench_{name}.dat")).await.unwrap());
            });
        });

        // Setup: Write a compressed file
        let storage_compressed = rt.block_on(async {
            let s = Storage::builder()
                .root(temp.path())
                .create(true)
                .compression(Compression::Lz4)
                .connect()
                .await
                .unwrap();
            s.write(format!("read_bench_{name}_compressed.dat"), &data).await.unwrap();
            s
        });

        // Benchmark: Read with compression
        group.bench_function(BenchmarkId::new("read_compressed", name), |b| {
            b.to_async(&rt).iter(|| async {
                black_box(
                    storage_compressed
                        .read(format!("read_bench_{name}_compressed.dat"))
                        .await
                        .unwrap(),
                );
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark: Namespace Operations
// ============================================================================

fn bench_namespace(c: &mut Criterion) {
    let mut group = c.benchmark_group("namespace");

    let temp = TempDir::new().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        Storage::builder().root(temp.path()).create(true).connect().await.unwrap()
    });

    group.bench_function("create_namespace", |b| {
        b.iter(|| {
            black_box(storage.namespace("test_ns").unwrap());
        });
    });

    let ns = storage.namespace("bench_ns").unwrap();
    let data = vec![42u8; 1024];

    group.bench_function("namespaced_write", |b| {
        b.to_async(&rt).iter(|| async {
            ns.write("test.dat", &data).await.unwrap();
        });
    });

    rt.block_on(async {
        ns.write("read_test.dat", &data).await.unwrap();
    });

    group.bench_function("namespaced_read", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(ns.read("read_test.dat").await.unwrap());
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark: Atomic Write Operations
// ============================================================================

fn bench_atomic_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_writes");
    group.measurement_time(Duration::from_secs(10));

    let temp = TempDir::new().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        Storage::builder().root(temp.path()).create(true).connect().await.unwrap()
    });

    let data = vec![42u8; 10 * 1024]; // 10KB

    group.bench_function("atomic_write_sync_all", |b| {
        b.to_async(&rt).iter(|| async {
            storage.write("atomic_test.dat", &data).await.unwrap();
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_path_resolution,
    bench_compression,
    bench_file_operations,
    bench_namespace,
    bench_atomic_writes,
);

criterion_main!(benches);
