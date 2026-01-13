//! Benchmarks for memory operations comparing sync vs async approaches.
//!
//! Run with: `cargo bench`
//! Or for async benchmarks: `cargo bench --features async`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use platform_mem::{FileMapped, Global, RawMem};

/// Benchmark sync memory allocation and growth using Global allocator
fn bench_sync_global_grow(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_global_grow");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut mem = Global::<u64>::new();
                mem.grow_filled(size, black_box(42u64)).unwrap();
                black_box(mem.allocated().len())
            });
        });
    }

    group.finish();
}

/// Benchmark sync memory allocation using FileMapped (mmap)
fn bench_sync_file_mapped_grow(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_file_mapped_grow");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("bench.bin");
                let mut mem = FileMapped::<u64>::from_path(&path).unwrap();
                unsafe {
                    mem.grow_zeroed(size).unwrap();
                }
                black_box(mem.allocated().len())
            });
        });
    }

    group.finish();
}

/// Benchmark sync random read access
fn bench_sync_random_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_random_read");

    for size in [1_000, 10_000, 100_000].iter() {
        let mut mem = Global::<u64>::new();
        mem.grow_filled(*size, 42u64).unwrap();

        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut idx = 0usize;
            b.iter(|| {
                // Pseudo-random access pattern
                idx = (idx * 1103515245 + 12345) % size;
                black_box(mem.allocated()[idx])
            });
        });
    }

    group.finish();
}

/// Benchmark sync sequential write access
fn bench_sync_sequential_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_sequential_write");

    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut mem = Global::<u64>::new();
            mem.grow_filled(size, 0u64).unwrap();

            b.iter(|| {
                for (i, slot) in mem.allocated_mut().iter_mut().enumerate() {
                    *slot = black_box(i as u64);
                }
                black_box(mem.allocated().len())
            });
        });
    }

    group.finish();
}

/// Benchmark grow/shrink cycles
fn bench_sync_grow_shrink_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_grow_shrink_cycle");

    for size in [100, 1_000, 10_000].iter() {
        group.throughput(Throughput::Elements(*size as u64 * 2)); // grow + shrink
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut mem = Global::<u64>::new();
                // Grow
                mem.grow_filled(size, 1u64).unwrap();
                // Shrink half
                mem.shrink(size / 2).unwrap();
                // Grow again
                mem.grow_filled(size, 2u64).unwrap();
                black_box(mem.allocated().len())
            });
        });
    }

    group.finish();
}

// Async benchmarks (only when async feature is enabled)
#[cfg(feature = "async")]
mod async_benches {
    use super::*;
    use platform_mem::AsyncFileMem;
    use tokio::runtime::Runtime;

    /// Benchmark async file memory growth
    pub fn bench_async_file_mem_grow(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let mut group = c.benchmark_group("async_file_mem_grow");

        for size in [100, 1_000, 10_000, 100_000].iter() {
            group.throughput(Throughput::Elements(*size as u64));
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();
                        mem.grow_filled(size, black_box(42u64)).await.unwrap();
                        black_box(mem.len())
                    })
                });
            });
        }

        group.finish();
    }

    /// Benchmark async file memory with persistence (sync to disk)
    pub fn bench_async_file_mem_with_sync(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let mut group = c.benchmark_group("async_file_mem_with_sync");

        for size in [100, 1_000, 10_000].iter() {
            group.throughput(Throughput::Elements(*size as u64));
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();
                        mem.grow_filled(size, black_box(42u64)).await.unwrap();
                        mem.sync().await.unwrap();
                        black_box(mem.len())
                    })
                });
            });
        }

        group.finish();
    }

    /// Benchmark async random read (from in-memory buffer)
    pub fn bench_async_random_read(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let mut group = c.benchmark_group("async_random_read");

        for size in [1_000, 10_000, 100_000].iter() {
            let mem = rt.block_on(async {
                let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();
                mem.grow_filled(*size, 42u64).await.unwrap();
                mem
            });

            group.throughput(Throughput::Elements(1000));
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
                let mut idx = 0usize;
                b.iter(|| {
                    // Pseudo-random access pattern
                    idx = (idx * 1103515245 + 12345) % size;
                    black_box(mem.get(idx).unwrap())
                });
            });
        }

        group.finish();
    }

    /// Compare sync FileMapped vs async AsyncFileMem for file persistence
    pub fn bench_compare_sync_async_persistence(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let mut group = c.benchmark_group("compare_persistence");

        for size in [1_000, 10_000].iter() {
            // Sync FileMapped (includes mmap overhead)
            group.bench_with_input(
                BenchmarkId::new("sync_filemapped", size),
                size,
                |b, &size| {
                    b.iter(|| {
                        let dir = tempfile::tempdir().unwrap();
                        let path = dir.path().join("sync.bin");
                        let mut mem = FileMapped::<u64>::from_path(&path).unwrap();
                        unsafe {
                            mem.grow_zeroed(size).unwrap();
                        }
                        for (i, slot) in mem.allocated_mut().iter_mut().enumerate() {
                            *slot = i as u64;
                        }
                        // FileMapped syncs on drop
                        black_box(mem.allocated().len())
                    });
                },
            );

            // Async file memory
            group.bench_with_input(
                BenchmarkId::new("async_filemem", size),
                size,
                |b, &size| {
                    b.iter(|| {
                        rt.block_on(async {
                            let mut mem = AsyncFileMem::<u64>::temp().await.unwrap();
                            mem.grow_filled(size, 0u64).await.unwrap();
                            for (i, slot) in mem.as_slice_mut().iter_mut().enumerate() {
                                *slot = i as u64;
                            }
                            mem.sync().await.unwrap();
                            black_box(mem.len())
                        })
                    });
                },
            );
        }

        group.finish();
    }
}

#[cfg(feature = "async")]
criterion_group!(
    async_benches_group,
    async_benches::bench_async_file_mem_grow,
    async_benches::bench_async_file_mem_with_sync,
    async_benches::bench_async_random_read,
    async_benches::bench_compare_sync_async_persistence,
);

criterion_group!(
    sync_benches,
    bench_sync_global_grow,
    bench_sync_file_mapped_grow,
    bench_sync_random_read,
    bench_sync_sequential_write,
    bench_sync_grow_shrink_cycle,
);

#[cfg(feature = "async")]
criterion_main!(sync_benches, async_benches_group);

#[cfg(not(feature = "async"))]
criterion_main!(sync_benches);
