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

fn traditional_filter(data: &[TraditionalRecord]) -> (f64, usize) {
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

fn soa_filter(store: &OptimizedRecordStore) -> (f64, usize) {
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

fn traditional_aggregation(data: &[TraditionalRecord]) -> Vec<f64> {
    let mut category_sums = vec![0.0; 10];
    for record in data {
        if record.status == Status::Active {
            category_sums[record.category as usize] += record.value;
        }
    }
    category_sums
}

fn soa_aggregation(store: &OptimizedRecordStore) -> Vec<f64> {
    let mut category_sums = vec![0.0; 10];
    for record in store.kernel().iter() {
        if *record.status == Status::Active {
            category_sums[*record.category as usize] += *record.value;
        }
    }
    category_sums
}

fn main() {
    println!("🔬 Performance Demo: Array of Structs vs Structure of Arrays");
    println!("===========================================================\n");
    println!("💡 For actual benchmarking, run: cargo bench\n");

    let sizes = vec![10_000, 100_000, 1_000_000];

    for &size in &sizes {
        println!("📊 Dataset size: {} records", size);
        println!(
            "Memory per record: {} bytes (AoS) vs columnar layout (SoA)",
            std::mem::size_of::<TraditionalRecord>()
        );

        // Generate test data
        print!("  Generating data... ");
        let traditional_data = generate_traditional_data(size);
        let soa_data = generate_soa_data(size);
        println!("✅");

        // Benchmark 1: Filter and sum values where status == Active
        println!("\n  🎯 Demo 1: Filter by status and sum values");
        let (trad_sum, trad_count) = traditional_filter(&traditional_data);
        let (soa_sum, soa_count) = soa_filter(&soa_data);

        // Verify results are identical
        assert_eq!(trad_count, soa_count);
        assert!((trad_sum - soa_sum).abs() < 0.001);
        println!(
            "     Results: {} active records, sum = {:.2}",
            trad_count, trad_sum
        );

        // Demo 2: Aggregate values by category for active records
        println!("\n  📈 Demo 2: Aggregate by category (active records only)");
        let trad_agg = traditional_aggregation(&traditional_data);
        let soa_agg = soa_aggregation(&soa_data);

        // Verify results are identical
        for (i, (&trad_val, &soa_val)) in trad_agg.iter().zip(soa_agg.iter()).enumerate() {
            assert!(
                (trad_val - soa_val).abs() < 0.001,
                "Category {} mismatch: {} vs {}",
                i,
                trad_val,
                soa_val
            );
        }
        println!(
            "     Results verified: aggregation by {} categories",
            trad_agg.len()
        );

        println!("\n  🚀 Why SoA can be faster:");
        println!("     • Better CPU cache utilization (fewer cache misses)");
        println!("     • Accessing only required fields (status, value, category)");
        println!("     • Reduced memory bandwidth usage");
        println!("     • More opportunities for CPU prefetching");
        println!("     • Run 'cargo bench' to see actual performance measurements");

        if size < 1_000_000 {
            println!("\n{}", "─".repeat(60));
        }
    }

    println!("\n🎯 Key Takeaways:");
    println!("  • SoA performance advantage increases with data size");
    println!("  • Filtering operations show the biggest improvements");
    println!("  • Cache-friendly access patterns reduce memory latency");
    println!("  • Business logic remains unchanged - only data layout differs");

    println!("\n💡 In real applications:");
    println!("  • Analytics queries can be 2-4x faster");
    println!("  • Reduced memory usage due to better data locality");
    println!("  • SIMD operations become more feasible");
    println!("  • Better scalability for large datasets");

    println!("\n✨ The framework gives you SoA performance with DDD APIs!");
}
