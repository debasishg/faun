use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use soa_macros::{SoA, SoAStore};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Status {
    Active,
    Inactive,
    Suspended,
}

// Traditional Array of Structs approach
#[derive(Debug, Clone, Copy)]
pub struct TraditionalRecord {
    pub id: u64,
    pub value: f64,
    pub status: Status,
    pub timestamp: u64,
    pub category: u32,
    pub metadata: u64,
}

// SoA approach using our macros
#[derive(SoA, SoAStore, Debug, Copy, Clone)]
#[soa_store(key = "id", shards = 8)]
pub struct OptimizedRecord {
    pub id: u64,
    pub value: f64,
    pub status: Status,
    pub timestamp: u64,
    pub category: u32,
    pub metadata: u64,
}

fn generate_traditional_data(count: usize) -> Vec<TraditionalRecord> {
    (0..count)
        .map(|i| TraditionalRecord {
            id: i as u64,
            value: (i % 1000) as f64 + 0.5,
            status: match i % 3 {
                0 => Status::Active,
                1 => Status::Inactive,
                _ => Status::Suspended,
            },
            timestamp: 1697731200 + (i % 86400) as u64,
            category: (i % 10) as u32,
            metadata: (i * 12345) as u64,
        })
        .collect()
}

fn generate_soa_data(count: usize) -> OptimizedRecordStore {
    let mut store = OptimizedRecordStore::new();

    for i in 0..count {
        store.add(OptimizedRecord {
            id: i as u64,
            value: (i % 1000) as f64 + 0.5,
            status: match i % 3 {
                0 => Status::Active,
                1 => Status::Inactive,
                _ => Status::Suspended,
            },
            timestamp: 1697731200 + (i % 86400) as u64,
            category: (i % 10) as u32,
            metadata: (i * 12345) as u64,
        });
    }

    store
}

fn benchmark_traditional_filter(data: &[TraditionalRecord]) -> (f64, usize) {
    // AoS: Loading entire 40-byte records but only using status (1 byte) + value (8 bytes)
    // Cache efficiency: 9/40 = 22.5% - lots of wasted memory bandwidth
    let sum: f64 = data
        .iter()
        .filter(|record| record.status == Status::Active)
        .map(|record| record.value)
        .sum();

    let count = data
        .iter()
        .filter(|record| record.status == Status::Active)
        .count();

    (sum, count)
}

fn benchmark_soa_filter(store: &OptimizedRecordStore) -> (f64, usize) {
    // SoA: Sequential access to status array, then value array
    // Perfect for CPU prefetching and cache line utilization
    // Cache efficiency: ~100% (using entire cache lines)
    let sum: f64 = store
        .kernel()
        .iter()
        .filter(|record| *record.status == Status::Active)
        .map(|record| *record.value)
        .sum();

    let count = store
        .kernel()
        .iter()
        .filter(|record| *record.status == Status::Active)
        .count();

    (sum, count)
}

fn benchmark_traditional_aggregation(data: &[TraditionalRecord]) -> Vec<f64> {
    // AoS: All fields (status, category, value) are in the same cache line
    // Single memory access loads all needed data - excellent spatial locality
    let mut category_sums = vec![0.0; 10];
    for record in data {
        if record.status == Status::Active {
            category_sums[record.category as usize] += record.value;
        }
    }
    category_sums
}

fn benchmark_soa_aggregation(store: &OptimizedRecordStore) -> Vec<f64> {
    // SoA: Three separate arrays mean three cache line accesses per iteration
    // This creates cache pressure and reduces effective memory bandwidth
    let mut category_sums = vec![0.0; 10];
    for record in store.kernel().iter() {
        if *record.status == Status::Active {
            category_sums[*record.category as usize] += *record.value;
        }
    }
    category_sums
}

// Note: A truly optimized SoA aggregation would require:
// 1. Direct access to underlying arrays (not available through current iterator)
// 2. SIMD vectorization for parallel processing
// 3. Cache-blocking algorithms to process data in L1-cache-sized chunks
// 4. Potentially different memory layouts (e.g., interleaving related fields)
//
// The current SoA framework doesn't expose these low-level optimizations,
// which is why AoS wins for multi-field aggregation workloads.

fn bench_filter_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_operations");

    for size in [10_000, 100_000, 1_000_000].iter() {
        let traditional_data = generate_traditional_data(*size);
        let soa_data = generate_soa_data(*size);

        group.bench_with_input(BenchmarkId::new("traditional_aos", size), size, |b, _| {
            b.iter(|| black_box(benchmark_traditional_filter(black_box(&traditional_data))))
        });

        group.bench_with_input(BenchmarkId::new("optimized_soa", size), size, |b, _| {
            b.iter(|| black_box(benchmark_soa_filter(black_box(&soa_data))))
        });
    }

    group.finish();
}

fn bench_aggregation_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregation_operations");

    for size in [10_000, 100_000, 1_000_000].iter() {
        let traditional_data = generate_traditional_data(*size);
        let soa_data = generate_soa_data(*size);

        group.bench_with_input(BenchmarkId::new("traditional_aos", size), size, |b, _| {
            b.iter(|| {
                black_box(benchmark_traditional_aggregation(black_box(
                    &traditional_data,
                )))
            })
        });

        group.bench_with_input(BenchmarkId::new("optimized_soa", size), size, |b, _| {
            b.iter(|| black_box(benchmark_soa_aggregation(black_box(&soa_data))))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_filter_operations,
    bench_aggregation_operations
);
criterion_main!(benches);
