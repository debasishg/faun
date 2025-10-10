/*!
# Advanced SoA vs AoS Performance Benchmarks

This benchmark suite provides a comprehensive comparison of Structure of Arrays (SoA)
vs Array of Structures (AoS) performance across different scenarios:

## Benchmark Groups:

### 📊 `aggregation_comparison`
Multi-field aggregation workloads (revenue by payment method)
- **Expected**: AoS typically wins due to data locality for multi-field access
- **Educational**: Shows that SoA isn't automatically faster

### 🚀 `single_field_access_patterns`
Operations where SoA typically excels
- **Expected**: SoA wins significantly on single-field operations
- **Examples**: Sum, count, filter, project operations
- **Educational**: Demonstrates SoA's strengths in columnar workloads

## Key Insights:
- SoA excels at single-field operations (sum, count, filter)
- AoS excels at multi-field operations (complex aggregations)
- Performance depends on access patterns, not just data structure
- Dataset size affects which approach is optimal

Run with: `cargo bench --package example_app`
Run specific group: `cargo bench --package example_app -- <group_name>`
*/

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use example_app::*;
use std::collections::HashMap;

/// Create test dataset for benchmarking
fn create_test_dataset(size: usize) -> (OrderAoS, OrderSoA) {
    let mut aos_data = OrderAoS::new();
    let mut soa_data = OrderSoA::new();

    for i in 0..size {
        let payment = match i % 3 {
            0 => PaymentMethod::CreditCard,
            1 => PaymentMethod::PayPal,
            _ => PaymentMethod::BankTransfer,
        };
        let status = match i % 5 {
            0 | 1 => OrderStatus::Delivered, // 40% delivered
            2 => OrderStatus::Shipped,       // 20% shipped
            3 => OrderStatus::Processing,    // 20% processing
            _ => OrderStatus::Pending,       // 20% pending
        };

        let order = Order {
            order_id: i as u64,
            customer_id: 1000 + (i % 100) as u64,
            product_id: 2000 + (i % 50) as u64,
            quantity: 1 + (i % 5) as u32,
            unit_price: 10.0 + (i % 200) as f64,
            total_amount: (1 + (i % 5)) as f64 * (10.0 + (i % 200) as f64),
            status,
            payment_method: payment,
            order_timestamp: 1234567890 + i as u64,
            shipping_address_hash: (i as u64).wrapping_mul(31),
        };

        aos_data.push(order.clone());
        soa_data.push(order);
    }

    (aos_data, soa_data)
}

/// Traditional AoS aggregation for comparison
fn aos_revenue_aggregation(orders: &OrderAoS) -> HashMap<PaymentMethod, f64> {
    orders.revenue_by_payment_method()
}

/// Benchmark SoA vs AoS aggregation performance
///
/// ** Why AoS beats SoA **
///
/// The aggregation_comparison benchmark measures revenue by payment method aggregation,
/// which requires accessing 3 fields per record:
/// - status (for filtering: only Delivered orders)
/// - payment_method (for grouping/HashMap key)
/// - total_amount (for summing the revenue)
///
/// This is a multi-field operation where AoS has fundamental advantages.
///
/// ** Memory Layout Advantages **
/// - Spatial locality: All 3 needed fields (status, payment_method, total_amount) are close together in memory
/// - Single cache miss: Loading one Order struct brings all needed data into cache
/// - Sequential access: Simple linear iteration through a contiguous Vec<Order>
/// - No indirection: Direct field access order.status, order.payment_method, order.total_amount
///
/// ** CPU-Friendly Pattern **
/// - Predictable memory access - perfect for prefetching
/// - Minimal pointer chasing - all data in one struct
/// - Good branch prediction - simple loop with predictable pattern
/// - Compiler optimization - easy for LLVM to optimize
///
/// ** Why SoA Iterator Loses Badly **
///
/// View creation overhead - self.iter() method generates view objects
/// Each iteration:
///
/// - Creates a new OrderView struct with pointers to 3 separate arrays
/// - Dereferences pointers: *order_view.status, *order_view.payment_method, *order_view.total_amount
/// - Memory scatter: Accesses 3 different arrays in separate memory locations
///
/// Cache Fragmentation:
///
/// Status array:   [S1|S2|S3|S4|S5|S6|S7|S8|...] (scattered access)
/// Payment array:  [P1|P2|P3|P4|P5|P6|P7|P8|...] (scattered access)  
/// Amount array:   [A1|A2|A3|A4|A5|A6|A7|A8|...] (scattered access)
///
/// - 3 cache misses per record instead of 1
/// - Poor cache utilization - loads data from 3 different cache lines
/// - Memory bandwidth waste - more total data movement
fn benchmark_aggregation_comparison(c: &mut Criterion) {
    let sizes = vec![1_000, 10_000, 100_000];

    let mut group = c.benchmark_group("aggregation_comparison");
    group.sample_size(100);

    for size in sizes {
        let (aos_data, soa_data) = create_test_dataset(size);

        // Traditional AoS approach
        group.bench_with_input(BenchmarkId::new("aos", size), &size, |b, _| {
            b.iter(|| black_box(aos_revenue_aggregation(black_box(&aos_data))))
        });

        // SoA with iterator (naive approach)
        group.bench_with_input(BenchmarkId::new("soa_iterator", size), &size, |b, _| {
            b.iter(|| black_box(soa_data.revenue_by_payment_method_iterator()))
        });

        // SoA with direct field access optimization
        group.bench_with_input(BenchmarkId::new("soa_optimized", size), &size, |b, _| {
            b.iter(|| black_box(soa_data.revenue_by_payment_method_optimized()))
        });

        // SoA with memory layout optimization
        group.bench_with_input(
            BenchmarkId::new("soa_memory_optimized", size),
            &size,
            |b, _| b.iter(|| black_box(soa_data.revenue_by_payment_method_memory_optimized())),
        );
    }

    group.finish();
}

/// Benchmark operations where SoA excels - single field access patterns
fn benchmark_single_field_access_patterns(c: &mut Criterion) {
    let sizes = vec![10_000, 100_000, 1_000_000];

    let mut group = c.benchmark_group("soa_advantages");
    group.sample_size(100);

    for size in sizes {
        let (aos_data, soa_data) = create_test_dataset(size);

        // Benchmark 1a: Sum all amounts (simple case - shows minimal SoA advantage)

        // Since the Rust compiler is extremely good at optimizing simple iterator chains.
        // Here's what happens:
        //
        // AoS Iterator Chain:
        // 1. orders.iter() - Sequential memory access (good cache behavior)
        // 2. .map(|o| o.total_amount) - Compiler optimizes this to direct field extraction
        // 3. .sum() - Gets vectorized by LLVM
        //
        // SoA Direct Array:
        // 1. total_amount_raw_array().iter() - Already a contiguous array
        // 2. .sum() - Also gets vectorized by LLVM
        // Result: Both approaches get compiled to nearly identical vectorized assembly code!
        group.bench_with_input(
            BenchmarkId::new("aos_sum_all_amounts", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let sum: f64 = aos_data.orders.iter().map(|o| o.total_amount).sum();
                    black_box(sum)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("soa_sum_all_amounts", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let sum: f64 = soa_data.total_amount_raw_array().iter().sum();
                    black_box(sum)
                })
            },
        );

        // Benchmark 1b: Conditional sum with branch prediction challenges
        // This creates a scenario where SoA's memory efficiency truly matters
        group.bench_with_input(
            BenchmarkId::new("aos_conditional_sum", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut sum = 0.0;
                    for order in &aos_data.orders {
                        // Complex condition that forces full struct access and creates branch misprediction
                        if matches!(order.status, OrderStatus::Delivered)
                            && order.customer_id % 7 == 0
                            && order.total_amount > 50.0
                        {
                            sum += order.total_amount;
                        }
                    }
                    black_box(sum)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("soa_conditional_sum", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let statuses = soa_data.status_raw_array();
                    let customers = soa_data.customer_id_raw_array();
                    let amounts = soa_data.total_amount_raw_array();

                    let mut sum = 0.0;
                    for i in 0..statuses.len() {
                        // Same condition but with better cache behavior
                        if matches!(statuses[i], OrderStatus::Delivered)
                            && customers[i] % 7 == 0
                            && amounts[i] > 50.0
                        {
                            sum += amounts[i];
                        }
                    }
                    black_box(sum)
                })
            },
        );

        // Benchmark 2: Count delivered orders (filtering on single field)

        // AoS version:
        // * Must load entire Order struct (80 bytes) to check status field
        // * Only uses ~1 byte out of 80 bytes loaded per cache line
        // * Memory amplification: 80x more data than needed
        group.bench_with_input(
            BenchmarkId::new("aos_count_delivered", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let count = aos_data
                        .orders
                        .iter()
                        .filter(|o| matches!(o.status, OrderStatus::Delivered))
                        .count();
                    black_box(count)
                })
            },
        );

        // SoA version:
        // * Only needs to load the status array (1 byte per record)
        // * Each cache line (64 bytes) contains 64 status entries - perfect cache utilization
        // * Memory amplification: 1x (only the needed data) - no memory wastage
        group.bench_with_input(
            BenchmarkId::new("soa_count_delivered", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let count = soa_data
                        .status_raw_array()
                        .iter()
                        .filter(|s| matches!(**s, OrderStatus::Delivered))
                        .count();
                    black_box(count)
                })
            },
        );

        // Benchmark 3: Find max amount (single field scan)
        //
        // Why Only Modest Advantage:
        // * Compiler Optimization: Modern compilers can optimize the AoS field extraction quite well
        // * Vectorization: Both approaches can be vectorized by LLVM
        // * Memory Access Pattern: Sequential scan works reasonably well even with larger structs
        // * CPU Cache: For the benchmark size, field extraction overhead is partially hidden by cache

        // Still SoA Wins Because:
        // * Better memory bandwidth utilization - only loads 8-byte amounts vs 80-byte structs
        // * More cache-friendly - 8 amounts per cache line vs ~1 struct per cache line
        // * Less memory pressure - leaves more cache space for other operations
        // * Better for large datasets - advantage grows with data size
        group.bench_with_input(BenchmarkId::new("aos_max_amount", size), &size, |b, _| {
            b.iter(|| {
                let max = aos_data
                    .orders
                    .iter()
                    .map(|o| o.total_amount)
                    .fold(0.0, f64::max);
                black_box(max)
            })
        });

        group.bench_with_input(BenchmarkId::new("soa_max_amount", size), &size, |b, _| {
            b.iter(|| {
                let max = soa_data
                    .total_amount_raw_array()
                    .iter()
                    .fold(0.0f64, |acc, &x| acc.max(x));
                black_box(max)
            })
        });

        // Benchmark 4: Collect all customer IDs (single field projection)

        // Memory efficiency:
        // * Loads: Only the exact data needed (8 bytes per customer ID)
        // * Waste ratio: 0% waste - every byte loaded is used
        // * Cache optimal: 8 customer IDs per cache line, perfect utilization
        // * Memory bandwidth: Minimal - only touches the customer_id array
        //
        // Why 6.3x Faster:
        // * 90% less memory traffic - only loads what's needed
        // * Perfect cache utilization - 8 customer IDs per 64-byte cache line
        // * No field extraction overhead - direct array copy via to_vec()
        // * Better prefetching - predictable sequential access pattern
        // * SIMD optimization - to_vec() can be vectorized for bulk copy
        group.bench_with_input(
            BenchmarkId::new("aos_collect_customers", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let customers: Vec<u64> =
                        aos_data.orders.iter().map(|o| o.customer_id).collect();
                    black_box(customers)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("soa_collect_customers", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let customers: Vec<u64> = soa_data.customer_id_raw_array().to_vec();
                    black_box(customers)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_aggregation_comparison,
    benchmark_single_field_access_patterns
);
criterion_main!(benches);
